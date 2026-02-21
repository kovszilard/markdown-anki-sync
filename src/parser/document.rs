use crate::types::{Block, MarkdownDocument};
use nom::{IResult, Parser, branch::alt, combinator::opt, multi::many0};

pub fn parse_document(input: &str) -> IResult<&str, MarkdownDocument> {
    let (input, front_matter) = opt(super::front_matter::parse_front_matter).parse(input)?;

    let (input, blocks) = many0(alt((
        |i| {
            let (i, metadata) = super::flashcard_metadata::parse_flashcard_metadata(i)?;
            let (i, blank) = opt(super::blank_line::parse_blank_line).parse(i)?;
            let (i, card) = super::flashcard::parse_flashcard(i)?;
            Ok((
                i,
                Block::FlashCardWithMeta {
                    metadata,
                    blank_line: blank,
                    flashcard: card,
                },
            ))
        },
        |i| {
            let (i, card) = super::flashcard::parse_flashcard(i)?;
            Ok((i, Block::FlashCard(card)))
        },
        |i| {
            let (i, block) = super::uninterested_block::parse_uninterested_block(i)?;
            Ok((i, Block::Uninterested(block)))
        },
    )))
    .parse(input)?;

    Ok((
        input,
        MarkdownDocument {
            front_matter,
            blocks,
        },
    ))
}

#[cfg(test)]
mod tests {
    use crate::types::FrontMatter;

    use super::*;
    use indoc::indoc;

    #[test]
    fn test_empty_document() {
        let input = "";
        let (rest, doc) = parse_document(input).unwrap();
        assert_eq!(rest, "");
        assert!(doc.front_matter.is_none());
        assert!(doc.blocks.is_empty());
    }

    #[test]
    fn test_front_matter_only() {
        let input = indoc! {"
            ---
            title: My Notes
            ---
        "};
        let (rest, doc) = parse_document(input).unwrap();
        assert_eq!(rest, "");
        assert!(doc.front_matter.is_some());
        assert!(doc.blocks.is_empty());
    }

    #[test]
    fn test_single_flashcard() {
        let input = indoc! {"
            ## Q: What is Rust?
            A systems programming language.
        "};
        let (rest, doc) = parse_document(input).unwrap();
        assert_eq!(rest, "");
        assert!(doc.front_matter.is_none());
        assert_eq!(doc.blocks.len(), 1);
        match &doc.blocks[0] {
            Block::FlashCard(card) => {
                assert_eq!(card.front, "What is Rust?");
                assert_eq!(card.back, "A systems programming language.\n");
            }
            _ => panic!("Expected Block::FlashCard"),
        }
    }

    #[test]
    fn test_metadata_flashcard_no_blank_line() {
        let input = indoc! {"
            <!-- anki_id: 123 -->
            ## Q: What is Rust?
            A systems programming language.
        "};
        let (rest, doc) = parse_document(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(doc.blocks.len(), 1);
        match &doc.blocks[0] {
            Block::FlashCardWithMeta {
                metadata,
                blank_line,
                flashcard,
            } => {
                assert_eq!(metadata.id, Some(123));
                assert!(blank_line.is_none());
                assert_eq!(flashcard.front, "What is Rust?");
            }
            _ => panic!("Expected Block::FlashCardWithMeta"),
        }
    }

    #[test]
    fn test_metadata_flashcard_with_blank_line() {
        let input = indoc! {"
            <!-- anki_id: 456 -->

            ## Q: What is Rust?
            A systems programming language.
        "};
        let (rest, doc) = parse_document(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(doc.blocks.len(), 1);
        match &doc.blocks[0] {
            Block::FlashCardWithMeta {
                metadata,
                blank_line,
                flashcard,
            } => {
                assert_eq!(metadata.id, Some(456));
                assert!(blank_line.is_some());
                assert_eq!(blank_line.as_ref().unwrap().raw, "\n");
                assert_eq!(flashcard.front, "What is Rust?");
            }
            _ => panic!("Expected Block::FlashCardWithMeta"),
        }
    }

    #[test]
    fn test_mixed_document() {
        let input = indoc! {"
            ---
            anki_sync:
              deck: TestDeck
            ---
            # Introduction

            Some intro text.

            <!-- anki_id: 1, anki_sync: true -->
            ## Q: What is Rust?
            A systems programming language.

            ## Q: What is Nom?
            A parser combinator library.
        "};
        let (rest, doc) = parse_document(input).unwrap();
        assert_eq!(rest, "");
        assert!(doc.front_matter.is_some());
        assert_eq!(doc.blocks.len(), 3);

        // Block 0: uninterested (intro text)
        assert!(matches!(&doc.blocks[0], Block::Uninterested(_)));

        // Block 1: metadata + flashcard
        match &doc.blocks[1] {
            Block::FlashCardWithMeta {
                metadata,
                flashcard,
                ..
            } => {
                assert_eq!(metadata.id, Some(1));
                assert_eq!(flashcard.front, "What is Rust?");
            }
            _ => panic!("Expected Block::FlashCardWithMeta"),
        }

        // Block 2: standalone flashcard (blank line between was consumed by previous card's raw)
        match &doc.blocks[2] {
            Block::FlashCard(card) => {
                assert_eq!(card.front, "What is Nom?");
            }
            _ => panic!("Expected Block::FlashCard"),
        }
    }

    #[test]
    fn test_two_flashcards_both_with_metadata() {
        let input = indoc! {"
            <!-- anki_id: 100, anki_deck: DeckA -->
            ## Q: What is Rust?
            A systems programming language.

            <!-- anki_id: 200, anki_deck: DeckB -->
            ## Q: What is Nom?
            A parser combinator library.
        "};
        let (rest, doc) = parse_document(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(doc.blocks.len(), 2);

        match &doc.blocks[0] {
            Block::FlashCardWithMeta {
                metadata,
                flashcard,
                ..
            } => {
                assert_eq!(metadata.id, Some(100));
                assert_eq!(metadata.deck.as_deref(), Some("DeckA"));
                assert_eq!(flashcard.front, "What is Rust?");
                assert_eq!(flashcard.back, "A systems programming language.\n");
            }
            _ => panic!("Expected Block::FlashCardWithMeta for first card"),
        }

        match &doc.blocks[1] {
            Block::FlashCardWithMeta {
                metadata,
                flashcard,
                ..
            } => {
                assert_eq!(metadata.id, Some(200));
                assert_eq!(metadata.deck.as_deref(), Some("DeckB"));
                assert_eq!(flashcard.front, "What is Nom?");
                assert_eq!(flashcard.back, "A parser combinator library.\n");
            }
            _ => panic!("Expected Block::FlashCardWithMeta for second card"),
        }
    }

    #[test]
    fn test_back_has_no_trailing_newlines_raw_preserves_them() {
        let input = indoc! {"
            <!-- anki_id: 100 -->
            ## Q: What is Rust?
            A systems programming language.

            <!-- anki_id: 200 -->
            ## Q: What is Nom?
            A parser combinator library.
        "};
        let (rest, doc) = parse_document(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(doc.blocks.len(), 2);

        match &doc.blocks[0] {
            Block::FlashCardWithMeta { flashcard, .. } => {
                assert_eq!(flashcard.back, "A systems programming language.\n");
                assert_eq!(
                    flashcard.raw,
                    "## Q: What is Rust?\nA systems programming language.\n\n"
                );
            }
            _ => panic!("Expected Block::FlashCardWithMeta for first card"),
        }

        match &doc.blocks[1] {
            Block::FlashCardWithMeta { flashcard, .. } => {
                assert_eq!(flashcard.back, "A parser combinator library.\n");
                assert_eq!(
                    flashcard.raw,
                    "## Q: What is Nom?\nA parser combinator library.\n"
                );
            }
            _ => panic!("Expected Block::FlashCardWithMeta for second card"),
        }
    }

    #[test]
    fn test_round_trip() {
        let input = indoc! {"
            ---
            anki_sync:
              deck: TestDeck
            ---
            # Introduction

            Some intro text.

            <!-- anki_id: 1, anki_sync: true -->
            ## Q: What is Rust?
            A systems programming language.

            ## Q: What is Nom?
            A parser combinator library.
        "};
        let (rest, doc) = parse_document(input).unwrap();
        assert_eq!(rest, "");

        let mut reconstructed = String::new();
        if let Some(ref fm) = doc.front_matter {
            match fm {
                FrontMatter::Raw { raw } | FrontMatter::AnkiSync { raw, .. } => {
                    reconstructed.push_str(raw);
                }
            }
        }
        for block in &doc.blocks {
            match block {
                Block::FlashCard(card) => {
                    reconstructed.push_str(&card.raw);
                }
                Block::FlashCardWithMeta {
                    metadata,
                    blank_line,
                    flashcard,
                } => {
                    reconstructed.push_str(&metadata.raw);
                    if let Some(bl) = blank_line {
                        reconstructed.push_str(&bl.raw);
                    }
                    reconstructed.push_str(&flashcard.raw);
                }
                Block::Uninterested(block) => {
                    reconstructed.push_str(&block.raw);
                }
            }
        }

        assert_eq!(reconstructed, input);
    }

    #[test]
    fn test_round_trip_with_regular_html_comment() {
        let input = indoc! {"
            # Introduction

            Some intro text.
            <!-- this is a regular HTML comment -->
            More text.

            <!-- anki_id: 1, anki_sync: true -->
            ## Q: What is Rust?
            A systems programming language.
        "};
        let (rest, doc) = parse_document(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(doc.blocks.len(), 2);
        assert!(matches!(&doc.blocks[0], Block::Uninterested(_)));
        assert!(matches!(&doc.blocks[1], Block::FlashCardWithMeta { .. }));

        let mut reconstructed = String::new();
        if let Some(ref fm) = doc.front_matter {
            match fm {
                FrontMatter::Raw { raw } | FrontMatter::AnkiSync { raw, .. } => {
                    reconstructed.push_str(raw);
                }
            }
        }
        for block in &doc.blocks {
            match block {
                Block::FlashCard(card) => {
                    reconstructed.push_str(&card.raw);
                }
                Block::FlashCardWithMeta {
                    metadata,
                    blank_line,
                    flashcard,
                } => {
                    reconstructed.push_str(&metadata.raw);
                    if let Some(bl) = blank_line {
                        reconstructed.push_str(&bl.raw);
                    }
                    reconstructed.push_str(&flashcard.raw);
                }
                Block::Uninterested(block) => {
                    reconstructed.push_str(&block.raw);
                }
            }
        }

        assert_eq!(reconstructed, input);
    }
}
