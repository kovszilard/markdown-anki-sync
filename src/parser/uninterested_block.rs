use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{line_ending, not_line_ending},
    combinator::{eof, not, recognize, rest},
    multi::many1,
};

use super::flashcard::parse_flashcard;

pub struct UninterestedBlock {
    pub raw: String,
}

fn non_special_line(input: &str) -> IResult<&str, &str> {
    let (input, _) = not(eof).parse(input)?;
    let (input, _) = not(parse_flashcard).parse(input)?;
    let (input, _) = not(tag::<&str, &str, nom::error::Error<&str>>("<!--")).parse(input)?;
    alt((recognize((not_line_ending, line_ending)), rest)).parse(input)
}

pub fn parse_uninterested_block(input: &str) -> IResult<&str, UninterestedBlock> {
    let (input, raw) = recognize(many1(non_special_line)).parse(input)?;
    Ok((
        input,
        UninterestedBlock {
            raw: raw.to_string(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_single_line() {
        let input = "Hello world\n";
        let (rest, block) = parse_uninterested_block(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(block.raw, "Hello world\n");
    }

    #[test]
    fn test_multi_line_with_blank_lines() {
        let input = indoc! {"
            Some text here.

            More text after a blank line.
            And another line.
        "};
        let (rest, block) = parse_uninterested_block(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(block.raw, input);
    }

    #[test]
    fn test_stops_before_flashcard_header() {
        let input = indoc! {"
            Regular content.
            ## Q: What is Rust?
            Answer here.
        "};
        let (rest, block) = parse_uninterested_block(input).unwrap();
        assert_eq!(block.raw, "Regular content.\n");
        assert_eq!(rest, "## Q: What is Rust?\nAnswer here.\n");
    }

    #[test]
    fn test_stops_before_metadata_comment() {
        let input = indoc! {"
            Regular content.
            <!-- anki_id: 123 -->
        "};
        let (rest, block) = parse_uninterested_block(input).unwrap();
        assert_eq!(block.raw, "Regular content.\n");
        assert_eq!(rest, "<!-- anki_id: 123 -->\n");
    }

    #[test]
    fn test_regular_header_consumed() {
        let input = indoc! {"
            ## Not Q: just a header
            Some body text.
        "};
        let (rest, block) = parse_uninterested_block(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(block.raw, input);
    }

    #[test]
    fn test_eof_without_trailing_newline() {
        let input = "Just some text";
        let (rest, block) = parse_uninterested_block(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(block.raw, "Just some text");
    }

    #[test]
    fn test_blank_lines_preserved() {
        let input = "\n\n\n";
        let (rest, block) = parse_uninterested_block(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(block.raw, "\n\n\n");
    }

    #[test]
    fn test_stops_before_h1_flashcard() {
        let input = indoc! {"
            Intro text.
            # Q: Top level question
            Answer.
        "};
        let (rest, block) = parse_uninterested_block(input).unwrap();
        assert_eq!(block.raw, "Intro text.\n");
        assert_eq!(rest, "# Q: Top level question\nAnswer.\n");
    }

    #[test]
    fn test_rejects_flashcard_at_start() {
        let input = "## Q: Question\nAnswer\n";
        assert!(parse_uninterested_block(input).is_err());
    }

    #[test]
    fn test_rejects_metadata_at_start() {
        let input = "<!-- anki_id: 1 -->";
        assert!(parse_uninterested_block(input).is_err());
    }
}
