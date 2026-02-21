use crate::types::FlashCardMetaData;

impl FlashCardMetaData {
    pub fn from_fields(
        id: Option<u64>,
        deck: Option<String>,
        sync: Option<bool>,
        tags: Option<Vec<String>>,
    ) -> Self {
        let anki_id = id.map_or_else(|| "".to_string(), |id| format!("anki_id: {}", id));
        let anki_deck = deck.clone().map_or_else(
            || "".to_string(),
            |deck| {
                if deck.contains(" ") {
                    format!("anki_deck: \"{}\"", deck)
                } else {
                    format!("anki_deck: {}", deck)
                }
            },
        );
        let anki_sync = sync.map_or_else(|| "".to_string(), |sync| format!("anki_sync: {}", sync));
        let anki_tags = tags.clone().map_or_else(
            || "".to_string(),
            |tags| {
                let formatted_tags: Vec<String> = tags
                    .into_iter()
                    .map(|tag| {
                        if tag.contains(" ") {
                            format!("\"{}\"", tag)
                        } else {
                            tag
                        }
                    })
                    .collect();
                format!("anki_tags: [{}]", formatted_tags.join(", "))
            },
        );

        let formatted_fields = vec![anki_id, anki_deck, anki_sync, anki_tags]
            .into_iter()
            .filter(|field| field != "")
            .collect::<Vec<String>>()
            .join(", ");

        let raw = format!("<!-- {} -->\n", formatted_fields);
        FlashCardMetaData {
            raw,
            id,
            deck,
            sync,
            tags,
        }
    }
}
