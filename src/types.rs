#[derive(Debug, Clone)]
pub enum Block {
    FlashCard(FlashCard),
    FlashCardWithMeta {
        metadata: FlashCardMetaData,
        blank_line: Option<BlankLine>,
        flashcard: FlashCard,
    },
    Uninterested(UninterestedBlock),
}

pub struct MarkdownDocument {
    pub front_matter: Option<FrontMatter>,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
pub struct FlashCard {
    pub raw: String,
    pub front: String,
    pub back: String,
}

#[derive(Debug, Clone)]
pub struct BlankLine {
    pub raw: String,
}

#[derive(Debug, Clone)]
pub struct FlashCardMetaData {
    pub raw: String,
    pub id: Option<u64>,
    pub sync: Option<bool>,
    pub deck: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FrontMatter {
    Raw {
        raw: String,
    },
    AnkiSync {
        raw: String,
        deck: Option<String>,
        tags: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub struct UninterestedBlock {
    pub raw: String,
}
