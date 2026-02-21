use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{line_ending, not_line_ending, space0, space1},
    combinator::{eof, not, recognize, rest, verify},
    multi::many1,
};

#[derive(Debug, Clone)]
pub struct FlashCard {
    pub raw: String,
    pub front: String,
    pub back: String,
}

fn parse_header_hashes(input: &str) -> IResult<&str, u8> {
    let (input, hashes) = verify(take_while1(|c| c == '#'), |s: &str| s.len() <= 6).parse(input)?;
    Ok((input, hashes.len() as u8))
}

fn parse_front(input: &str) -> IResult<&str, (&str, u8)> {
    let (input, level) = parse_header_hashes(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, _) = tag("Q:").parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, front_text) = verify(not_line_ending, |s: &str| !s.is_empty()).parse(input)?;
    let (input, _) = alt((recognize(line_ending), eof)).parse(input)?;
    Ok((input, (front_text, level)))
}

fn terminating_header(header_level: u8) -> impl FnMut(&str) -> IResult<&str, ()> {
    move |input: &str| {
        let (input, hashes) = take_while1(|c| c == '#').parse(input)?;
        if hashes.len() > header_level as usize {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Verify,
            )));
        }
        let (input, _) = tag(" ").parse(input)?;
        Ok((input, ()))
    }
}

fn non_terminator_line(header_level: u8) -> impl FnMut(&str) -> IResult<&str, &str> {
    move |input: &str| {
        let (input, _) = not(eof).parse(input)?;
        let (input, _) = not(terminating_header(header_level)).parse(input)?;
        let (input, _) = not(super::flashcard_metadata::parse_flashcard_metadata).parse(input)?;
        alt((recognize((not_line_ending, line_ending)), rest)).parse(input)
    }
}

fn parse_back(input: &str, header_level: u8) -> IResult<&str, &str> {
    recognize(many1(non_terminator_line(header_level))).parse(input)
}

/// Trims leading and trailing blank lines from a string, preserving interior blank lines.
/// Returns a sub-slice of the input (zero-copy).
fn trim_blank_lines(s: &str) -> &str {
    // Find first non-whitespace character, then back up to start of its line
    let start = match s.find(|c: char| !c.is_whitespace()) {
        Some(pos) => s[..pos].rfind('\n').map_or(0, |nl| nl + 1),
        None => return "",
    };

    // Find last non-whitespace character, then advance to end of its line (including \n)
    let end = match s.rfind(|c: char| !c.is_whitespace()) {
        Some(pos) => {
            // unwrap is safe: pos is the start of a valid char from rfind
            let after = pos + s[pos..].chars().next().unwrap().len_utf8();
            s[after..].find('\n').map_or(s.len(), |nl| after + nl + 1)
        }
        None => return "",
    };

    &s[start..end]
}

pub fn parse_flashcard(input: &str) -> IResult<&str, FlashCard> {
    let start = input;
    let (input, (front_text, header_level)) = parse_front(input)?;
    let (input, back_text) = parse_back(input, header_level)?;
    let trimmed_back = trim_blank_lines(back_text);
    if trimmed_back.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        )));
    }
    let raw = &start[..start.len() - input.len()];
    Ok((
        input,
        FlashCard {
            raw: raw.to_string(),
            front: front_text.to_string(),
            back: trimmed_back.to_string(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_parse_header_hashes_valid() {
        assert_eq!(parse_header_hashes("# rest"), Ok((" rest", 1)));
        assert_eq!(parse_header_hashes("## rest"), Ok((" rest", 2)));
        assert_eq!(parse_header_hashes("###### rest"), Ok((" rest", 6)));
    }

    #[test]
    fn test_parse_header_hashes_too_many() {
        assert!(parse_header_hashes("####### rest").is_err());
    }

    #[test]
    fn test_basic_flashcard() {
        let input = indoc! {"
            ## Q: What is Rust?
            Rust is a systems programming language.
            It focuses on safety and performance.
        "};
        let (rest, card) = parse_flashcard(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(card.front, "What is Rust?");
        assert_eq!(
            card.back,
            indoc! {"
                Rust is a systems programming language.
                It focuses on safety and performance.
            "}
        );
        assert_eq!(card.raw, input);
    }

    #[test]
    fn test_terminated_by_same_level_header() {
        let input = indoc! {"
            ## Q: First question
            Answer to first.
            ## Next header
        "};
        let (rest, card) = parse_flashcard(input).unwrap();
        assert_eq!(card.front, "First question");
        assert_eq!(card.back, "Answer to first.\n");
        assert_eq!(rest, "## Next header\n");
        assert_eq!(card.raw, "## Q: First question\nAnswer to first.\n");
    }

    #[test]
    fn test_terminated_by_higher_level_header() {
        let input = indoc! {"
            ### Q: Deep question
            Some answer.
            ## Higher level header
        "};
        let (rest, card) = parse_flashcard(input).unwrap();
        assert_eq!(card.front, "Deep question");
        assert_eq!(card.back, "Some answer.\n");
        assert_eq!(rest, "## Higher level header\n");
    }

    #[test]
    fn test_deeper_headers_included_in_back() {
        let input = indoc! {"
            ## Q: Question with sub-headers
            Some text.
            ### Sub-header
            More text.
            #### Even deeper
            Final text.
        "};
        let (rest, card) = parse_flashcard(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(card.front, "Question with sub-headers");
        assert_eq!(
            card.back,
            indoc! {"
                Some text.
                ### Sub-header
                More text.
                #### Even deeper
                Final text.
            "}
        );
    }

    #[test]
    fn test_empty_back_rejected() {
        let input = indoc! {"
            ## Q: Empty answer
            ## Next card
        "};
        assert!(parse_flashcard(input).is_err());
    }

    #[test]
    fn test_front_at_eof_no_back_rejected() {
        let input = "## Q: Just a question";
        assert!(parse_flashcard(input).is_err());
    }

    #[test]
    fn test_empty_front_rejected() {
        let input = "## Q:\nSome back\n";
        assert!(parse_flashcard(input).is_err());
    }

    #[test]
    fn test_whitespace_only_front_rejected() {
        let input = "## Q: \nSome back\n";
        assert!(parse_flashcard(input).is_err());
    }

    #[test]
    fn test_no_trailing_newline_at_eof() {
        let input = "## Q: Question\nAnswer without trailing newline";
        let (rest, card) = parse_flashcard(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(card.front, "Question");
        assert_eq!(card.back, "Answer without trailing newline");
        assert_eq!(card.raw, input);
    }

    #[test]
    fn test_q_without_space_after_colon() {
        let input = "## Q:No space\nBody\n";
        let (rest, card) = parse_flashcard(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(card.front, "No space");
        assert_eq!(card.back, "Body\n");
    }

    #[test]
    fn test_single_hash_header() {
        let input = "# Q: Top level\nAnswer\n";
        let (rest, card) = parse_flashcard(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(card.front, "Top level");
        assert_eq!(card.back, "Answer\n");
    }

    #[test]
    fn test_non_q_header_rejected() {
        let input = "## Not a question\nBody\n";
        assert!(parse_flashcard(input).is_err());
    }

    #[test]
    fn test_back_with_interior_blank_lines_preserved() {
        let input = indoc! {"
            ## Q: Blank lines
            First paragraph.

            Second paragraph.

            Third paragraph.
        "};
        let (rest, card) = parse_flashcard(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            card.back,
            indoc! {"
                First paragraph.

                Second paragraph.

                Third paragraph.
            "}
        );
    }

    #[test]
    fn test_header_level_above_6_rejected() {
        let input = "####### Q: Too deep\nBody\n";
        assert!(parse_flashcard(input).is_err());
    }

    #[test]
    fn test_leading_blank_lines_stripped_from_back() {
        let input = indoc! {"
            ## Q: Leading blanks


            Actual answer.
        "};
        let (rest, card) = parse_flashcard(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(card.back, "Actual answer.\n");
        assert_eq!(card.raw, input);
    }

    #[test]
    fn test_trailing_blank_lines_stripped_from_back() {
        let input = indoc! {"
            ## Q: Trailing blanks
            Actual answer.


        "};
        let (rest, card) = parse_flashcard(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(card.back, "Actual answer.\n");
        assert_eq!(card.raw, input);
    }

    #[test]
    fn test_leading_and_trailing_blank_lines_stripped_interior_preserved() {
        let input = indoc! {"
            ## Q: Both ends trimmed

            First paragraph.

            Second paragraph.

        "};
        let (rest, card) = parse_flashcard(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            card.back,
            indoc! {"
                First paragraph.

                Second paragraph.
            "}
        );
        assert_eq!(card.raw, input);
    }

    #[test]
    fn test_back_with_only_blank_lines_rejected() {
        let input = indoc! {"
            ## Q: Blank back



            ## Next header
        "};
        assert!(parse_flashcard(input).is_err());
    }
}
