use crate::anki::{BasicModelFields, Note};
use crate::parser::document::Block;
use crate::parser::flashcard::FlashCard;
use crate::parser::flashcard_metadata::FlashCardMetaData;
use crate::parser::front_matter::FrontMatter;

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
                    tags: vec!["tag1".to_string(), "tag2".to_string()],
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
            } if sync.is_none_or(|x| x == true) => {
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
}
