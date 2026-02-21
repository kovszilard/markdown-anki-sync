use notes_to_anki::anki::*;
use notes_to_anki::document_with_anki_actions::AnkiAction;
use notes_to_anki::document_with_anki_actions::BlockWithAnkiAction;
use notes_to_anki::document_with_anki_actions::MarkdownDocumentWithAnkiActions;
use notes_to_anki::parser::document::parse_document;
use notes_to_anki::types::MarkdownDocument;
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

    let (new_result_blocks, created_count, updated_count) = document_with_anki_actions
        .blocks_with_anki_action
        .iter()
        .fold(
            (Vec::new(), 0u32, 0u32),
            |(mut blocks, mut created, mut updated), block_with_action| {
                let request = anki_action_to_request_payload(block_with_action);
                let response = match request {
                    Some(request) => {
                        let result = ureq::post("http://localhost:8765")
                            .send_json(&request)
                            .map(|mut body| body.body_mut().read_json::<Response>())
                            .flatten();
                        result.map_or_else(|_| None, Some)
                    }
                    None => None,
                };
                match block_with_action.sync_with_anki_response(&response) {
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

    println!("Created: {}, Updated: {}", created_count, updated_count);

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

    std::fs::write(filename, synced_document.raw())?;
    Ok(())
}
