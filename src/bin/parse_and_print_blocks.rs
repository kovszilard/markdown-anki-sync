use notes_to_anki::parser::document::parse_document;
use notes_to_anki::types::Block;
use notes_to_anki::types::FrontMatter;
use std::env;
use std::process;

fn main() {
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

    if let Some(ref fm) = doc.front_matter {
        match fm {
            FrontMatter::Raw { .. } => println!("FrontMatter: Raw"),
            FrontMatter::AnkiSync { deck, tags, .. } => {
                println!("FrontMatter: AnkiSync (deck: {:?}, tags: {:?})", deck, tags);
            }
        }
    }

    for block in &doc.blocks {
        match block {
            Block::FlashCard(card) => {
                println!("FlashCard: {}", card.front);
            }
            Block::FlashCardWithMeta {
                metadata,
                flashcard,
                ..
            } => {
                println!(
                    "FlashCardWithMeta: id={:?}, sync={:?}, deck={:?}, tags={:?} | {}",
                    metadata.id, metadata.sync, metadata.deck, metadata.tags, flashcard.front
                );
            }
            Block::Passthrough(_) => {
                println!("Passthrough");
            }
        }
    }

    if !rest.is_empty() {
        eprintln!("Warning: unparsed remaining input ({} bytes)", rest.len());
    }
}
