use notes_to_anki::anki::Params;
use notes_to_anki::anki::Request;
use notes_to_anki::anki::Response;
use notes_to_anki::document_with_anki_actions::AnkiAction;
use notes_to_anki::document_with_anki_actions::BlockWithAnkiAction;
use notes_to_anki::document_with_anki_actions::MarkdownDocumentWithAnkiActions;
use notes_to_anki::parser::document::parse_document;
use std::env;
use std::process;

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
            .into_iter()
            .map(|block| BlockWithAnkiAction::from_block(block, &doc.front_matter))
            .collect();

        MarkdownDocumentWithAnkiActions {
            front_matter: doc.front_matter,
            blocks_with_anki_action: new_blocks,
        }
    };

    for request in document_with_anki_actions
        .blocks_with_anki_action
        .iter()
        .filter_map(|block_with_action| anki_action_to_request_payload(block_with_action))
    {
        println!("{}", serde_json::to_string(&request).unwrap());
        let response = ureq::post("http://localhost:8765")
            .send_json(&request)?
            .body_mut()
            .read_json::<Response>();
        println!("Response: {:?}", response);
    }

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
