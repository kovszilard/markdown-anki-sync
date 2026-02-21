use crate::anki::{BasicModelFields, Note, Params, Request, Response};
use crate::types::{BlankLine, Block, FlashCard, FlashCardMetaData, FrontMatter, MarkdownDocument};

#[derive(Debug)]
pub struct BlockWithAnkiAction {
    pub block: Block,
    pub anki_action: AnkiAction,
}

#[derive(Debug)]
pub enum AnkiAction {
    CreateNote(Note),
    UpdateNote(Note),
    DoNothing,
}

#[derive(Debug)]
pub struct MarkdownDocumentWithAnkiActions {
    pub front_matter: Option<FrontMatter>,
    pub blocks_with_anki_action: Vec<BlockWithAnkiAction>,
}

impl MarkdownDocumentWithAnkiActions {
    pub fn from_document(doc: MarkdownDocument) -> Self {
        let blocks_with_anki_action = doc
            .blocks
            .iter()
            .map(|block| BlockWithAnkiAction::from_block(block.clone(), &doc.front_matter))
            .collect();

        Self {
            front_matter: doc.front_matter.clone(),
            blocks_with_anki_action,
        }
    }

    pub fn sync(
        self,
        send_request: impl Fn(&Request) -> Option<Response>,
    ) -> Result<(MarkdownDocument, u32, u32), String> {
        let block_count = self.blocks_with_anki_action.len();

        let (blocks, created, updated) = self.blocks_with_anki_action.iter().fold(
            (Vec::new(), 0u32, 0u32),
            |(mut blocks, mut created, mut updated), block_with_action| {
                let request = block_with_action.to_request_payload();
                let response = request.as_ref().and_then(&send_request);
                match block_with_action.block_from_response(&response) {
                    Ok(block) => {
                        match &block_with_action.anki_action {
                            AnkiAction::CreateNote(_) => created += 1,
                            AnkiAction::UpdateNote(_) => updated += 1,
                            AnkiAction::DoNothing => {}
                        }
                        blocks.push(block);
                    }
                    Err(err) => {
                        eprintln!("Error syncing block: {}", err);
                    }
                }
                (blocks, created, updated)
            },
        );

        if blocks.len() == block_count {
            Ok((
                MarkdownDocument {
                    front_matter: self.front_matter,
                    blocks,
                },
                created,
                updated,
            ))
        } else {
            Err("Error syncing blocks with Anki responses.".to_string())
        }
    }
}

impl BlockWithAnkiAction {
    pub fn from_block(block: Block, front_matter: &Option<FrontMatter>) -> Self {
        let mut default_deck: Option<String> = None;
        let mut default_tags: Vec<String> = Vec::new();

        if let Some(FrontMatter::AnkiSync { raw: _, deck, tags }) = front_matter {
            default_deck = deck.clone();
            default_tags = tags.clone();
        }

        match block {
            ref block @ Block::FlashCard(FlashCard {
                raw: _,
                ref front,
                ref back,
            }) => {
                let note = Note {
                    id: None,
                    deck_name: default_deck.unwrap_or("Default".to_string()),
                    model_name: "Basic".to_string(),
                    fields: BasicModelFields {
                        front: front.clone(),
                        back: back.clone(),
                    },
                    tags: default_tags.clone(),
                };

                Self {
                    block: block.clone(),
                    anki_action: AnkiAction::CreateNote(note),
                }
            }

            ref block @ Block::FlashCardWithMeta {
                metadata:
                    FlashCardMetaData {
                        raw: _,
                        id: Some(id),
                        sync,
                        ref deck,
                        ref tags,
                    },
                blank_line: _,
                flashcard:
                    FlashCard {
                        raw: _,
                        ref front,
                        ref back,
                    },
            } if sync.is_none_or(|x| x) => {
                let deck_name = deck
                    .clone()
                    .or(default_deck)
                    .unwrap_or("Default".to_string());
                let tags = tags.clone().unwrap_or(default_tags);

                let note = Note {
                    id: Some(id),
                    deck_name,
                    model_name: "Basic".to_string(),
                    fields: BasicModelFields {
                        front: front.clone(),
                        back: back.clone(),
                    },
                    tags,
                };

                Self {
                    block: block.clone(),

                    anki_action: AnkiAction::UpdateNote(note),
                }
            }

            other => Self {
                block: other,
                anki_action: AnkiAction::DoNothing,
            },
        }
    }

    pub fn block_from_response(&self, response: &Option<Response>) -> Result<Block, String> {
        match response {
            Some(response) => match self {
                // Create a note from flashcard
                BlockWithAnkiAction {
                    block: Block::FlashCard(FlashCard { raw, front, back }),
                    anki_action: AnkiAction::CreateNote(_),
                } if response.result.is_some() && response.error.is_none() => {
                    let id = response.result.unwrap();
                    Ok(Block::FlashCardWithMeta {
                        metadata: FlashCardMetaData::from_fields(Some(id), None, None, None),
                        blank_line: Some(BlankLine::single()),
                        flashcard: FlashCard {
                            raw: raw.clone(),
                            front: front.clone(),
                            back: back.clone(),
                        },
                    })
                }
                // Create a note from flashcard with metadata
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
                // Update note from flashcard with metadata
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
                _ => Err(format!(
                    "Unexpected block or Anki response. Block: {:#?}, Response: {:#?}",
                    self.block, response
                )),
            },
            _ => Ok(self.block.clone()),
        }
    }

    pub fn to_request_payload(&self) -> Option<Request> {
        match &self.anki_action {
            AnkiAction::CreateNote(note) => Some(Request {
                action: "addNote".to_string(),
                version: 6,
                params: Params { note: note.clone() },
            }),
            AnkiAction::UpdateNote(note) => Some(Request {
                action: "updateNote".to_string(),
                version: 6,
                params: Params { note: note.clone() },
            }),
            AnkiAction::DoNothing => None,
        }
    }
}
