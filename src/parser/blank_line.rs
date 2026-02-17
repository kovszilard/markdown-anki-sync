use nom::{
    IResult, Parser,
    character::complete::{line_ending, space0},
    combinator::recognize,
    multi::many1,
};

#[derive(Clone)]
pub struct BlankLine {
    pub raw: String,
}

pub fn parse_blank_line(input: &str) -> IResult<&str, BlankLine> {
    let (input, raw) = recognize(many1((space0, line_ending))).parse(input)?;
    Ok((
        input,
        BlankLine {
            raw: raw.to_string(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_single_empty_line() {
        let (rest, bl) = parse_blank_line("\n").unwrap();
        assert_eq!(rest, "");
        assert_eq!(bl.raw, "\n");
    }

    #[test]
    fn test_single_whitespace_only_line_with_spaces() {
        let (rest, bl) = parse_blank_line("   \n").unwrap();
        assert_eq!(rest, "");
        assert_eq!(bl.raw, "   \n");
    }

    #[test]
    fn test_single_whitespace_only_line_with_tab() {
        let (rest, bl) = parse_blank_line("\t\n").unwrap();
        assert_eq!(rest, "");
        assert_eq!(bl.raw, "\t\n");
    }

    #[test]
    fn test_single_whitespace_only_line_with_mixed_spaces_and_tabs() {
        let (rest, bl) = parse_blank_line("  \t  \n").unwrap();
        assert_eq!(rest, "");
        assert_eq!(bl.raw, "  \t  \n");
    }

    #[test]
    fn test_multiple_blank_lines() {
        let (rest, bl) = parse_blank_line("\n\n\n").unwrap();
        assert_eq!(rest, "");
        assert_eq!(bl.raw, "\n\n\n");
    }

    #[test]
    fn test_mixed_empty_and_whitespace_only_lines() {
        let input = indoc! {"

            \x20\x20\x20

        "};
        let (rest, bl) = parse_blank_line(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(bl.raw, input);
    }

    #[test]
    fn test_windows_line_endings() {
        let (rest, bl) = parse_blank_line("\r\n").unwrap();
        assert_eq!(rest, "");
        assert_eq!(bl.raw, "\r\n");
    }

    #[test]
    fn test_stops_before_non_blank_content() {
        let input = "\n\nsome content\n";
        let (rest, bl) = parse_blank_line(input).unwrap();
        assert_eq!(rest, "some content\n");
        assert_eq!(bl.raw, "\n\n");
    }

    #[test]
    fn test_rejects_non_blank_content() {
        assert!(parse_blank_line("some content\n").is_err());
    }
}
