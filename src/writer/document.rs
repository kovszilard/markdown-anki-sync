use crate::types::{Block, FrontMatter, MarkdownDocument};

impl MarkdownDocument {
    pub fn raw(&self) -> String {
        let mut final_raw = String::new();
        if let Some(ref fm) = self.front_matter {
            match fm {
                FrontMatter::Raw { raw } | FrontMatter::AnkiSync { raw, .. } => {
                    final_raw.push_str(raw);
                }
            }
        }
        for block in &self.blocks {
            match block {
                Block::FlashCard(card) => {
                    final_raw.push_str(&card.raw);
                }
                Block::FlashCardWithMeta {
                    metadata,
                    blank_line,
                    flashcard,
                } => {
                    final_raw.push_str(&metadata.raw);
                    if let Some(bl) = blank_line {
                        final_raw.push_str(&bl.raw);
                    }
                    final_raw.push_str(&flashcard.raw);
                }
                Block::Passthrough(block) => {
                    final_raw.push_str(&block.raw);
                }
            }
        }
        final_raw
    }
}
