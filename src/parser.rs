use regex::Regex;
use std::{error::Error, fs::File, io::Read};

struct FrontMatter {
    deck: String,
    tags: Vec<String>,
}

enum TextPosition {
    FrontMatterPosition {
        start_line: u32,
        end_line: Option<u32>,
    },
    FlashCardMetadataPosition(u32),
    FlashCardPosition {
        start_line: u32,
        end_line: Option<u32>,
    },
}

// <!-- anki_sync: false, anki_id: 12345, anki_deck: Default, anki_tags: [tag1, tag2, ...] -->
struct FlashCardMetadata {
    id: Option<u32>,
    sync: Option<bool>,
}

struct FlashCard {
    front: String,
    back: String,
}

// parsed format
enum FlashCardInMarkdown {
    FlashCard {
        flashcard: FlashCard,
        flashcard_start_position: u32,
        flashcard_end_position: u32,
    },
    FlashCardWithMetadata {
        metadata: FlashCardMetadata,
        metadata_position: u32,
        flashcard: FlashCard,
        flashcard_start_position: u32,
        flashcard_end_position: u32,
    },
}

// created in anki format (FlashCardMetadata has id)
enum FlashCardSyncedToAnki {
    FlashCardWithoutMetadataPosition {
        metadata: FlashCardMetadata,
        flashcard: FlashCard,
        flashcard_start_position: u32,
        flashcard_end_position: u32,
    },
    FlashCardWitMetadataPosition {
        metadata: FlashCardMetadata,
        metadata_position: u32,
        flashcard: FlashCard,
        flashcard_start_position: u32,
        flashcard_end_position: u32,
    },
}

// ready to write back to markdown format
// FlashCardInMarkdown::FlashCardWithMetadata

struct MarkdonwDocument {
    front_matter: Option<FrontMatter>,
    flashcards: Vec<FlashCardInMarkdown>,
}

fn get_front_matter(input: &str) -> Option<&str> {
    let trimmed = input.trim_start();
    let rest = trimmed.strip_prefix("---")?;
    let rest = rest
        .strip_prefix('\n')
        .or_else(|| rest.strip_prefix("\r\n"))?;
    let end = rest.find("\n---")?;
    Some(rest[..end].trim_end_matches('\r'))
}

fn parse_falashcard_metadata_comment(line: &str) -> Option<FlashCardMetadata> {
    let re = Regex::new(
        r#"(?x)
            ^\s*<!--\s*
            (?:
                (?:anki_sync\s*:\s*(?P<anki_sync>true|false)\s*,?\s*)|
                (?:anki_id\s*:\s*(?P<anki_id>\d+)\s*,?\s*)|
                (?:anki_deck\s*:\s*(?P<anki_deck>\w+)\s*,?\s*)
            )+
            -->\s*$
        "#,
    )
    .unwrap();

    match re.captures(line) {
        Some(caps) => {
            let id = caps.name("anki_id").map(|a| a.as_str().to_string());
            let sync = caps.name("anki_sync").map(|a| a.as_str().to_string());

            let id = id?.parse::<u32>().ok();
            let sync = sync?.parse::<bool>().ok();

            Some(FlashCardMetadata { id, sync })
        }
        None => None,
    }
}

fn parse_flashcard_in_markdown(lines: &str) -> Option<FlashCard> {
    let mut lines_iter = lines.lines();
    let front = lines_iter.next()?.strip_prefix("###### Q:");
    let back = lines_iter.collect::<Vec<&str>>().join("\n");
    front.map(|f| FlashCard {
        front: f.trim().to_string(),
        back: back.to_string(),
    })
}

fn parse_a<A>(line: &str) -> Option<A> {
    todo!()
}

fn parse_b<A>(lines: Vec<&str>) -> Option<A> {
    todo!()
}

fn parse_c<A>(lines: Vec<&str>) -> Option<A> {
    todo!()
}

// fn process_file(path: &str) -> Result<(), Box<dyn Error>> {
//     let lines: Vec<&str> = std::fs::read_to_string(path)?.lines().collect();
//
//     // lines.iter().enumerate().
//     lines.lines.fold
//     Ok(())
// }

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_get_front_matter() {
        let input = indoc! {"
            ---
            key: value
            ---
            # Body
        "};
        assert_eq!(get_front_matter(input), Some("key: value"));
    }

    #[test]
    fn test_get_front_matter_no_frontmatter() {
        let input = indoc! {"
            # Just a heading
            Some text
        "};
        assert_eq!(get_front_matter(input), None);
    }

    #[test]
    fn test_get_front_matter_multiple_lines() {
        let input = indoc! {"
            ---
            key1: value1
            key2: value2
            ---
            # Body
        "};
        assert_eq!(get_front_matter(input), Some("key1: value1\nkey2: value2"));
    }

    #[test]
    fn test_parse_falashcard_metadata_comment() {
        let line = "<!-- anki_id: 12345, anki_sync: false -->";
        let metadata = parse_falashcard_metadata_comment(line).unwrap();
        assert_eq!(metadata.id, Some(12345));
        assert_eq!(metadata.sync, Some(false));
    }

    #[test]
    fn test_parse_flashcard_in_markdown() {
        let input = indoc! {"
            ###### Q: What is the capital of France?
            Paris
        "};
        let flashcard = parse_flashcard_in_markdown(input).unwrap();
        assert_eq!(flashcard.front, "What is the capital of France?");
        assert_eq!(flashcard.back, "Paris");
    }

    // ...existing code...
}
