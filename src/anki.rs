use serde::{Deserialize, Serialize};

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
