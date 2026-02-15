use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::{alphanumeric1, line_ending, not_line_ending, space0, space1},
    combinator::{eof, opt, recognize},
    multi::{many0, many1, separated_list1},
    sequence::delimited,
};

#[derive(Debug, PartialEq)]
pub enum FrontMatter {
    Raw {
        raw: String,
    },
    AnkiSync {
        raw: String,
        deck: Option<String>,
        tags: Vec<String>,
    },
}

enum AnkiSyncField {
    Deck(String),
    Tags(Vec<String>),
}

// --- Delimiter and raw extraction ---

fn parse_front_matter_delimiter(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("---").parse(input)?;
    let (input, _) = alt((recognize(line_ending), eof)).parse(input)?;
    Ok((input, "---"))
}

fn parse_raw_front_matter(input: &str) -> IResult<&str, &str> {
    let start = input;
    let (input, _) = parse_front_matter_delimiter(input)?;
    let (input, _content) = take_until("---").parse(input)?;
    let (input, _) = parse_front_matter_delimiter(input)?;
    // Consume optional trailing newline after closing delimiter
    let (input, _) = opt(line_ending).parse(input)?;
    let raw = &start[..start.len() - input.len()];
    Ok((input, raw))
}

// --- Value parsers ---

fn parse_rest_of_line(input: &str) -> IResult<&str, &str> {
    let (input, line) = not_line_ending.parse(input)?;
    let (input, _) = alt((recognize(line_ending), eof)).parse(input)?;
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        )));
    }
    Ok((input, trimmed))
}

fn parse_word_or_quoted_string(input: &str) -> IResult<&str, &str> {
    alt((
        alphanumeric1,
        delimited(tag("\""), take_while(|c| c != '"' && c != '\n'), tag("\"")),
    ))
    .parse(input)
}

fn parse_flow_list(input: &str) -> IResult<&str, Vec<&str>> {
    delimited(
        (tag("["), space0),
        separated_list1((space0, tag(","), space0), parse_word_or_quoted_string),
        (space0, tag("]")),
    )
    .parse(input)
}

fn parse_block_list_item(input: &str) -> IResult<&str, &str> {
    let (input, _) = space1.parse(input)?;
    let (input, _) = tag("-").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, value) = parse_word_or_quoted_string(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, _) = alt((recognize(line_ending), eof)).parse(input)?;
    Ok((input, value))
}

fn parse_block_list(input: &str) -> IResult<&str, Vec<&str>> {
    many1(parse_block_list_item).parse(input)
}

// --- Field parsers ---

fn parse_deck_field(input: &str) -> IResult<&str, &str> {
    let (input, _) = space1.parse(input)?;
    let (input, _) = tag("deck:").parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, value) = parse_rest_of_line(input)?;
    Ok((input, value))
}

fn parse_tags_block_variant(input: &str) -> IResult<&str, Vec<&str>> {
    let (input, _) = alt((recognize(line_ending), eof)).parse(input)?;
    parse_block_list(input)
}

fn parse_tags_flow_variant(input: &str) -> IResult<&str, Vec<&str>> {
    let (input, list) = parse_flow_list(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, _) = alt((recognize(line_ending), eof)).parse(input)?;
    Ok((input, list))
}

fn parse_tags_field(input: &str) -> IResult<&str, Vec<&str>> {
    let (input, _) = space1.parse(input)?;
    let (input, _) = tag("tags:").parse(input)?;
    let (input, _) = space0.parse(input)?;
    alt((parse_tags_block_variant, parse_tags_flow_variant)).parse(input)
}

fn parse_anki_sync_field(input: &str) -> IResult<&str, AnkiSyncField> {
    alt((
        |i| parse_deck_field(i).map(|(r, v)| (r, AnkiSyncField::Deck(v.to_string()))),
        |i| {
            parse_tags_field(i).map(|(r, v)| {
                (
                    r,
                    AnkiSyncField::Tags(v.into_iter().map(String::from).collect()),
                )
            })
        },
    ))
    .parse(input)
}

// --- Skip helpers ---

fn is_indented_line(input: &str) -> bool {
    input.starts_with(' ') || input.starts_with('\t')
}

fn skip_indented_line(input: &str) -> IResult<&str, ()> {
    let (input, _) = take_while1(|c| c == ' ' || c == '\t').parse(input)?;
    let (input, _) = not_line_ending.parse(input)?;
    let (input, _) = alt((recognize(line_ending), eof)).parse(input)?;
    Ok((input, ()))
}

fn skip_non_anki_sync_line(input: &str) -> IResult<&str, ()> {
    // Must be a non-indented, non-empty line that doesn't start with "anki_sync:"
    if input.is_empty() || input.starts_with("anki_sync:") {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        )));
    }
    if is_indented_line(input) {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        )));
    }
    let (input, _) = not_line_ending.parse(input)?;
    let (input, _) = alt((recognize(line_ending), eof)).parse(input)?;
    // Skip any indented children
    let (input, _) = many0(skip_indented_line).parse(input)?;
    Ok((input, ()))
}

fn skip_blank_line(input: &str) -> IResult<&str, ()> {
    let (input, _) = line_ending.parse(input)?;
    Ok((input, ()))
}

fn skip_non_anki_sync_lines(input: &str) -> IResult<&str, ()> {
    let (input, _) = many0(alt((skip_non_anki_sync_line, skip_blank_line))).parse(input)?;
    Ok((input, ()))
}

// --- Main parser ---

fn parse_anki_sync_content(input: &str) -> IResult<&str, (Option<String>, Vec<String>)> {
    // Skip any non-anki_sync top-level keys
    let (input, _) = skip_non_anki_sync_lines(input)?;

    if input.is_empty() || !input.starts_with("anki_sync:") {
        return Ok((input, (None, vec![])));
    }

    // Consume the "anki_sync:" line
    let (input, _) = tag("anki_sync:").parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, _) = alt((recognize(line_ending), eof)).parse(input)?;

    // Parse indented fields under anki_sync
    let (input, fields) = many0(parse_anki_sync_field).parse(input)?;

    let (deck, tags) = fields
        .into_iter()
        .fold((None, vec![]), |(mut deck, mut tags), field| {
            match field {
                AnkiSyncField::Deck(d) => deck = Some(d),
                AnkiSyncField::Tags(t) => tags = t,
            }
            (deck, tags)
        });

    Ok((input, (deck, tags)))
}

pub fn parse_front_matter(input: &str) -> IResult<&str, FrontMatter> {
    let (remaining, raw) = parse_raw_front_matter(input)?;

    // Second pass: parse the YAML content (between the delimiters)
    let inner_start = raw.find('\n').map(|i| i + 1).unwrap_or(raw.len());
    let inner_end = raw.rfind("---").unwrap_or(raw.len());
    let inner = &raw[inner_start..inner_end];

    let (deck, tags) = match parse_anki_sync_content(inner) {
        Ok((_, (deck, tags))) => (deck, tags),
        Err(_) => (None, vec![]),
    };

    let raw_string = raw.to_string();

    if deck.is_some() || !tags.is_empty() {
        Ok((
            remaining,
            FrontMatter::AnkiSync {
                raw: raw_string,
                deck,
                tags,
            },
        ))
    } else {
        Ok((remaining, FrontMatter::Raw { raw: raw_string }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_deck_and_flow_tags() {
        let input = indoc! {"
            ---
            anki_sync:
              deck: My Deck Name
              tags: [tag1, tag2]
            ---
            rest
        "};
        let (rest, fm) = parse_front_matter(input).unwrap();
        assert_eq!(rest, "rest\n");
        match fm {
            FrontMatter::AnkiSync { deck, tags, .. } => {
                assert_eq!(deck.as_deref(), Some("My Deck Name"));
                assert_eq!(tags, vec!["tag1", "tag2"]);
            }
            _ => panic!("Expected AnkiSync"),
        }
    }

    #[test]
    fn test_deck_and_block_tags() {
        let input = indoc! {"
            ---
            anki_sync:
              deck: My Deck Name
              tags:
                - tag1
                - tag2
            ---
            rest
        "};
        let (rest, fm) = parse_front_matter(input).unwrap();
        assert_eq!(rest, "rest\n");
        match fm {
            FrontMatter::AnkiSync { deck, tags, .. } => {
                assert_eq!(deck.as_deref(), Some("My Deck Name"));
                assert_eq!(tags, vec!["tag1", "tag2"]);
            }
            _ => panic!("Expected AnkiSync"),
        }
    }

    #[test]
    fn test_only_deck_no_tags() {
        let input = indoc! {"
            ---
            anki_sync:
              deck: MyDeck
            ---
        "};
        let (rest, fm) = parse_front_matter(input).unwrap();
        assert_eq!(rest, "");
        match fm {
            FrontMatter::AnkiSync { deck, tags, .. } => {
                assert_eq!(deck.as_deref(), Some("MyDeck"));
                assert!(tags.is_empty());
            }
            _ => panic!("Expected AnkiSync"),
        }
    }

    #[test]
    fn test_only_flow_tags_no_deck() {
        let input = indoc! {"
            ---
            anki_sync:
              tags: [tag1, tag2]
            ---
        "};
        let (rest, fm) = parse_front_matter(input).unwrap();
        assert_eq!(rest, "");
        match fm {
            FrontMatter::AnkiSync { deck, tags, .. } => {
                assert_eq!(deck, None);
                assert_eq!(tags, vec!["tag1", "tag2"]);
            }
            _ => panic!("Expected AnkiSync"),
        }
    }

    #[test]
    fn test_only_block_tags_no_deck() {
        let input = indoc! {"
            ---
            anki_sync:
              tags:
                - alpha
                - beta
            ---
        "};
        let (rest, fm) = parse_front_matter(input).unwrap();
        assert_eq!(rest, "");
        match fm {
            FrontMatter::AnkiSync { deck, tags, .. } => {
                assert_eq!(deck, None);
                assert_eq!(tags, vec!["alpha", "beta"]);
            }
            _ => panic!("Expected AnkiSync"),
        }
    }

    #[test]
    fn test_deck_with_spaces() {
        let input = indoc! {"
            ---
            anki_sync:
              deck: My Long Deck Name With Spaces
            ---
        "};
        let (_, fm) = parse_front_matter(input).unwrap();
        match fm {
            FrontMatter::AnkiSync { deck, .. } => {
                assert_eq!(deck.as_deref(), Some("My Long Deck Name With Spaces"));
            }
            _ => panic!("Expected AnkiSync"),
        }
    }

    #[test]
    fn test_tags_with_quoted_strings_flow() {
        let input = indoc! {r#"
            ---
            anki_sync:
              tags: [tag1, "tag two", tag3]
            ---
        "#};
        let (_, fm) = parse_front_matter(input).unwrap();
        match fm {
            FrontMatter::AnkiSync { tags, .. } => {
                assert_eq!(tags, vec!["tag1", "tag two", "tag3"]);
            }
            _ => panic!("Expected AnkiSync"),
        }
    }

    #[test]
    fn test_tags_with_quoted_strings_block() {
        let input = indoc! {r#"
            ---
            anki_sync:
              tags:
                - tag1
                - "tag two"
                - tag3
            ---
        "#};
        let (_, fm) = parse_front_matter(input).unwrap();
        match fm {
            FrontMatter::AnkiSync { tags, .. } => {
                assert_eq!(tags, vec!["tag1", "tag two", "tag3"]);
            }
            _ => panic!("Expected AnkiSync"),
        }
    }

    #[test]
    fn test_unknown_keys_before_and_after() {
        let input = indoc! {"
            ---
            title: My Notes
            author: Someone
            anki_sync:
              deck: TestDeck
              tags: [a, b]
            other_key: value
            ---
        "};
        let (_, fm) = parse_front_matter(input).unwrap();
        match fm {
            FrontMatter::AnkiSync { deck, tags, .. } => {
                assert_eq!(deck.as_deref(), Some("TestDeck"));
                assert_eq!(tags, vec!["a", "b"]);
            }
            _ => panic!("Expected AnkiSync"),
        }
    }

    #[test]
    fn test_empty_front_matter_is_raw() {
        let input = "---\n---\n";
        let (rest, fm) = parse_front_matter(input).unwrap();
        assert_eq!(rest, "");
        match fm {
            FrontMatter::Raw { .. } => {}
            _ => panic!("Expected Raw"),
        }
    }

    #[test]
    fn test_no_anki_sync_block_is_raw() {
        let input = indoc! {"
            ---
            title: My Document
            author: Someone
            ---
        "};
        let (_, fm) = parse_front_matter(input).unwrap();
        match fm {
            FrontMatter::Raw { .. } => {}
            _ => panic!("Expected Raw"),
        }
    }

    #[test]
    fn test_anki_sync_with_no_deck_or_tags_is_raw() {
        let input = indoc! {"
            ---
            anki_sync:
            ---
        "};
        let (_, fm) = parse_front_matter(input).unwrap();
        match fm {
            FrontMatter::Raw { .. } => {}
            _ => panic!("Expected Raw"),
        }
    }

    #[test]
    fn test_raw_content_preservation() {
        let input = indoc! {"
            ---
            anki_sync:
              deck: MyDeck
            ---
            body
        "};
        let (_, fm) = parse_front_matter(input).unwrap();
        let expected_raw = indoc! {"
            ---
            anki_sync:
              deck: MyDeck
            ---
        "};
        match fm {
            FrontMatter::AnkiSync { raw, .. } => {
                assert_eq!(raw, expected_raw);
            }
            _ => panic!("Expected AnkiSync"),
        }
    }

    #[test]
    fn test_remaining_input_returned() {
        let input = indoc! {"
            ---
            anki_sync:
              deck: Test
            ---
            # Q: What is this?
            An answer.
        "};
        let (rest, _) = parse_front_matter(input).unwrap();
        assert_eq!(
            rest,
            indoc! {"
                # Q: What is this?
                An answer.
            "}
        );
    }

    #[test]
    fn test_missing_closing_delimiter_fails() {
        let input = indoc! {"
            ---
            anki_sync:
              deck: Test
        "};
        assert!(parse_front_matter(input).is_err());
    }

    #[test]
    fn test_unknown_key_with_indented_children_skipped() {
        let input = indoc! {"
            ---
            nested_thing:
              child1: value
              child2: value
            anki_sync:
              deck: Found
            ---
        "};
        let (_, fm) = parse_front_matter(input).unwrap();
        match fm {
            FrontMatter::AnkiSync { deck, .. } => {
                assert_eq!(deck.as_deref(), Some("Found"));
            }
            _ => panic!("Expected AnkiSync"),
        }
    }
}
