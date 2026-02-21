use crate::types::BlankLine;

impl BlankLine {
    pub fn empty() -> Self {
        BlankLine {
            raw: "\n".to_string(),
        }
    }
}
