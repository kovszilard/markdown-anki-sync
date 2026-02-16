use std::env;
use std::process;

use notes_to_anki::parser::document::{Block, parse_document};
use notes_to_anki::parser::front_matter::FrontMatter;

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
            FrontMatter::Raw { raw } | FrontMatter::AnkiSync { raw, .. } => {
                print!("{}", raw);
            }
        }
    }

    for block in &doc.blocks {
        match block {
            Block::FlashCard(card) => {
                print!("{}", card.raw);
            }
            Block::FlashCardWithMeta {
                metadata,
                blank_line,
                flashcard,
                ..
            } => {
                print!("{}", metadata.raw);
                if let Some(bl) = blank_line {
                    print!("{}", bl.raw);
                }
                print!("{}", flashcard.raw);
            }
            Block::Uninterested(block) => {
                print!("{}", block.raw);
            }
        }
    }

    if !rest.is_empty() {
        eprint!("{}", rest);
    }
}
