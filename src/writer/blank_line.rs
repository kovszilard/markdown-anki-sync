use crate::parser::blank_line::BlankLine;

impl BlankLine {
    pub fn empty() -> Self {
        BlankLine {
            raw: "\n".to_string(),
        }
    }
}
