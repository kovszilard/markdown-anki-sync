use notes_to_anki::anki::Params;
use notes_to_anki::anki::Request;
use notes_to_anki::anki::Response;
use notes_to_anki::document_with_anki_actions::AnkiAction;
use notes_to_anki::document_with_anki_actions::BlockWithAnkiAction;
use notes_to_anki::document_with_anki_actions::MarkdownDocumentWithAnkiActions;
use notes_to_anki::parser::blank_line::BlankLine;
use notes_to_anki::parser::document::Block;
use notes_to_anki::parser::document::MarkdownDocument;
use notes_to_anki::parser::document::parse_document;
use notes_to_anki::parser::flashcard::FlashCard;
use notes_to_anki::parser::flashcard_metadata::FlashCardMetaData;
use std::env;
use std::error::Error;
use std::process;

pub struct AppError(String);

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for AppError {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <markdown-file>", args[0]);
        process::exit(1);
    }

    let filename = &args[1];
    let contents = match std::fs::read_to_string(filename) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading '{}': {}", filename, e);
            process::exit(1);
        }
    };

    let (rest, doc) = match parse_document(&contents) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            process::exit(1);
        }
    };

    if !rest.is_empty() {
        eprintln!("Warning: unparsed remaining input ({} bytes)", rest.len());
    }

    let document_with_anki_actions: MarkdownDocumentWithAnkiActions = {
        let new_blocks: Vec<BlockWithAnkiAction> = doc
            .blocks
            .iter()
            .map(|block| BlockWithAnkiAction::from_block(block.clone(), &doc.front_matter))
            .collect();

        MarkdownDocumentWithAnkiActions {
            front_matter: doc.front_matter.clone(),
            blocks_with_anki_action: new_blocks,
        }
    };

    let new_result_blocks = document_with_anki_actions
        .blocks_with_anki_action
        .iter()
        .flat_map(|block_with_action| {
            let request = anki_action_to_request_payload(block_with_action);
            let response = match request {
                Some(request) => {
                    let result = ureq::post("http://localhost:8765")
                        .send_json(&request)
                        .map(|mut body| body.body_mut().read_json::<Response>())
                        .flatten();
                    result.map_or_else(
                        |err| {
                            eprintln!(
                                "Error sending request to AnkiConnect: {:?}, for block: {:#?}",
                                err, block_with_action.block
                            );
                            None
                        },
                        |ok| {
                            println!("Received response from AnkiConnect: {:?}", ok);
                            Some(ok)
                        },
                    )
                }
                None => None,
            };
            let updated_block = sync_block_with_anki_response(block_with_action, &response);
            updated_block
        })
        .collect::<Vec<_>>();
    dbg!(doc.blocks.len());
    dbg!(document_with_anki_actions.blocks_with_anki_action.len());
    dbg!(new_result_blocks.len());
    dbg!(doc.blocks.len());
    let is_all_ok = new_result_blocks.len() == doc.blocks.len();
    let synced_document: Option<MarkdownDocument> = if is_all_ok {
        Some(MarkdownDocument {
            front_matter: doc.front_matter,
            blocks: new_result_blocks,
        })
    } else {
        None
    };

    let synced_document = synced_document
        .ok_or_else(|| AppError("Error syncing blocks with Anki responses.".to_string()))?;

    std::fs::write("/tmp/anki-markdown.md", synced_document.raw())?;
    Ok(())
}

fn anki_action_to_request_payload(action: &BlockWithAnkiAction) -> Option<Request> {
    match action {
        BlockWithAnkiAction {
            block: _,
            anki_action: AnkiAction::CreateNote(note),
        } => Some(Request {
            action: "addNote".to_string(),
            version: 6,
            params: Params { note: note.clone() },
        }),
        BlockWithAnkiAction {
            block: _,
            anki_action: AnkiAction::UpdateNote(note),
        } => Some(Request {
            action: "UpdateNote".to_string(),
            version: 6,
            params: Params { note: note.clone() },
        }),
        BlockWithAnkiAction {
            block: _,
            anki_action: AnkiAction::DoNothing,
        } => None,
    }
}

fn sync_block_with_anki_response(
    block_with_anki_action: &BlockWithAnkiAction,
    response: &Option<Response>,
) -> Result<Block, AppError> {
    match response {
        Some(response) => match block_with_anki_action {
            // create note from flashcard
            BlockWithAnkiAction {
                block: Block::FlashCard(FlashCard { raw, front, back }),
                anki_action: AnkiAction::CreateNote(_),
            } if response.result.is_some() && response.error.is_none() => {
                let id = response.result.unwrap();
                Ok(Block::FlashCardWithMeta {
                    metadata: FlashCardMetaData::from_fields(Some(id), None, None, None),
                    blank_line: Some(BlankLine::empty()),
                    flashcard: FlashCard {
                        raw: raw.clone(),
                        front: front.clone(),
                        back: back.clone(),
                    },
                })
            }
            // create note from flashcard with metadata
            BlockWithAnkiAction {
                block:
                    Block::FlashCardWithMeta {
                        metadata,
                        blank_line,
                        flashcard,
                    },
                anki_action: AnkiAction::CreateNote(_),
            } if response.result.is_some() && response.error.is_none() => {
                let id = response.result.unwrap();
                Ok(Block::FlashCardWithMeta {
                    metadata: FlashCardMetaData::from_fields(
                        Some(id),
                        metadata.deck.clone(),
                        metadata.sync,
                        metadata.tags.clone(),
                    ),
                    blank_line: blank_line.clone(),
                    flashcard: FlashCard {
                        raw: flashcard.raw.clone(),
                        front: flashcard.front.clone(),
                        back: flashcard.back.clone(),
                    },
                })
            }
            // update note from flashcard with metadata
            BlockWithAnkiAction {
                block:
                    Block::FlashCardWithMeta {
                        metadata,
                        blank_line,
                        flashcard,
                    },
                anki_action: AnkiAction::UpdateNote(_),
            } if response.result.is_none() && response.error.is_none() => {
                Ok(Block::FlashCardWithMeta {
                    metadata: FlashCardMetaData::from_fields(
                        metadata.id,
                        metadata.deck.clone(),
                        metadata.sync,
                        metadata.tags.clone(),
                    ),
                    blank_line: blank_line.clone(),
                    flashcard: FlashCard {
                        raw: flashcard.raw.clone(),
                        front: flashcard.front.clone(),
                        back: flashcard.back.clone(),
                    },
                })
            }
            _ => Err(AppError(format!(
                "Unexpected block or Anki response. Block: {:#?}, Response: {:#?}",
                block_with_anki_action.block, response
            ))),
        },
        _ => Ok(block_with_anki_action.block.clone()),
    }
}
