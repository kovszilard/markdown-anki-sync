use crate::types::BlankLine;

impl BlankLine {
    pub fn single() -> Self {
        BlankLine {
            raw: "\n".to_string(),
        }
    }
}
