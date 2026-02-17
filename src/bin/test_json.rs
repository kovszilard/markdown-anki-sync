use notes_to_anki::anki::{BasicModelFields, Note, Params, Request};
use serde_json::json;

fn main() {
    let _request = Request {
        action: "addNote".to_string(),
        version: 6,
        params: Params {
            note: Note {
                id: None,
                deck_name: "Default".to_string(),
                model_name: "Basic".to_string(),
                fields: BasicModelFields {
                    front: "front content".to_string(),
                    back: "back content".to_string(),
                },
                tags: vec!["tag1".to_string(), "tag2".to_string()],
            },
        },
    };

    let request2 = json!({
        "action": "canAddNotes",
        "version": 6,
        "params": {
            "notes":[
                {
                    "deckName": "Default",
                    "modelName": "Basic",
                    "fields": {
                        "Front": "front content",
                        "Back": "back content"
                    },
                    "tags": ["tag1", "tag2"]
                }
            ]
        }
    });

    let json = serde_json::to_string_pretty(&request2).unwrap();
    println!("{json}");
}
