struct FrontMatter {
    deck: String,
    tags: Vec<String>,
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

struct MarkdonwDocument {
    front_matter: Option<FrontMatter>,
    flashcards: Vec<FlashCardInMarkdown>,
}

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

mod flashcard_metadata;
