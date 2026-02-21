use serde::{Deserialize, Serialize};

use crate::document_with_anki_actions::{AnkiAction, BlockWithAnkiAction};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub action: String,
    pub version: u64,
    pub params: Params,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Params {
    pub note: Note,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: Option<u64>,
    pub deck_name: String,
    pub model_name: String,
    pub fields: BasicModelFields,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BasicModelFields {
    #[serde(rename = "Front")]
    pub front: String,
    #[serde(rename = "Back")]
    pub back: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
    pub result: Option<u64>,
    pub error: Option<String>,
}

pub fn anki_action_to_request_payload(action: &BlockWithAnkiAction) -> Option<Request> {
    match action {
        BlockWithAnkiAction {
            block: _,
            anki_action: AnkiAction::CreateNote(note),
        } => Some(Request {
            action: "addNote".to_string(),
            version: 6,
            params: Params { note: note.clone() },
        }),
        BlockWithAnkiAction {
            block: _,
            anki_action: AnkiAction::UpdateNote(note),
        } => Some(Request {
            action: "updateNote".to_string(),
            version: 6,
            params: Params { note: note.clone() },
        }),
        BlockWithAnkiAction {
            block: _,
            anki_action: AnkiAction::DoNothing,
        } => None,
    }
}
