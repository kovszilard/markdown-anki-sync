use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::space0,
    character::complete::{alpha1, alphanumeric1, digit1},
    combinator::{map_res, value},
    multi::separated_list1,
    sequence::{delimited, preceded},
};

pub struct FlashCardMetaData {
    pub id: Option<u32>,
    pub sync: Option<bool>,
    pub deck: Option<String>,
    pub tags: Option<Vec<String>>,
}

enum Field<'a> {
    Id(u32),
    Sync(bool),
    Deck(&'a str),
    Tags(Vec<&'a str>),
}

// value parsers
pub fn parse_u32_digits(input: &str) -> IResult<&str, u32> {
    map_res(digit1, |s: &str| s.parse::<u32>()).parse(input)
}

pub fn parse_word_or_quoted_string(input: &str) -> IResult<&str, &str> {
    alt((
        alphanumeric1,
        delimited(tag("\""), take_while(|c| c != '"' && c != '\n'), tag("\"")),
    ))
    .parse(input)
}

pub fn parse_bool(input: &str) -> IResult<&str, bool> {
    alt((value(true, tag("true")), value(false, tag("false")))).parse(input)
}

// key: value pair parser
pub fn parse_key_value<'a, O>(
    key: &'a str,
    value_parser: impl Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>,
) -> impl Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>> {
    preceded((tag(key), space0, tag(":"), space0), value_parser)
}

// parsers for specific keys
pub fn parse_anki_id(input: &str) -> IResult<&str, u32> {
    parse_key_value("anki_id", parse_u32_digits).parse(input)
}

pub fn parse_anki_deck(input: &str) -> IResult<&str, &str> {
    parse_key_value("anki_deck", parse_word_or_quoted_string).parse(input)
}

pub fn parse_anki_sync(input: &str) -> IResult<&str, bool> {
    parse_key_value("anki_sync", parse_bool).parse(input)
}

pub fn parse_list(input: &str) -> IResult<&str, Vec<&str>> {
    delimited(
        (tag("["), space0),
        separated_list1((space0, tag(","), space0), parse_word_or_quoted_string),
        (space0, tag("]")),
    )
    .parse(input)
}

pub fn parse_anki_tags(input: &str) -> IResult<&str, Vec<&str>> {
    parse_key_value("anki_tags", parse_list).parse(input)
}

// parse specific fields
fn parse_field(input: &str) -> IResult<&str, Field> {
    alt((
        |i| parse_anki_id(i).map(|(r, v)| (r, Field::Id(v))),
        |i| parse_anki_sync(i).map(|(r, v)| (r, Field::Sync(v))),
        |i| parse_anki_deck(i).map(|(r, v)| (r, Field::Deck(v))),
        |i| parse_anki_tags(i).map(|(r, v)| (r, Field::Tags(v))),
    ))
    .parse(input)
}

// parse the entire metadata comment
pub fn parse_flashcard_metadata(input: &str) -> IResult<&str, FlashCardMetaData> {
    let (input, fields) = delimited(
        (tag("<!--"), space0),
        separated_list1((space0, tag(","), space0), parse_field),
        (space0, tag("-->")),
    )
    .parse(input)?;

    let mut metadata = FlashCardMetaData {
        id: None,
        sync: None,
        deck: None,
        tags: None,
    };

    for f in fields {
        match f {
            Field::Id(v) => metadata.id = Some(v),
            Field::Sync(v) => metadata.sync = Some(v),
            Field::Deck(v) => metadata.deck = Some(v.to_string()),
            Field::Tags(v) => metadata.tags = Some(v.into_iter().map(String::from).collect()),
        }
    }

    Ok((input, metadata))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_u32_digits() {
        let cases = [("1234", 1234u32), ("0", 0u32), ("987654321", 987654321u32)];
        for (input, expected) in cases.iter() {
            let (_, value) = parse_u32_digits(input).expect("Should parse u32");
            assert_eq!(value, *expected);
        }
    }

    #[test]
    fn test_all_fields() {
        let input = "<!-- anki_id: 1234, anki_sync: true, anki_deck: hello -->";
        let (rest, meta) = parse_flashcard_metadata(input).expect("Should parse");
        assert_eq!(rest, "");
        assert_eq!(meta.id, Some(1234));
        assert_eq!(meta.sync, Some(true));
        assert_eq!(meta.deck.as_deref(), Some("hello"));
    }

    #[test]
    fn test_different_order() {
        let input = "<!-- anki_deck: \"My Deck\", anki_id: 42, anki_sync: false -->";
        let (rest, meta) = parse_flashcard_metadata(input).expect("Should parse");
        assert_eq!(rest, "");
        assert_eq!(meta.id, Some(42));
        assert_eq!(meta.sync, Some(false));
        assert_eq!(meta.deck.as_deref(), Some("My Deck"));
    }

    #[test]
    fn test_partial_fields() {
        let input = "<!-- anki_sync: true -->";
        let (rest, meta) = parse_flashcard_metadata(input).expect("Should parse");
        assert_eq!(rest, "");
        assert_eq!(meta.id, None);
        assert_eq!(meta.sync, Some(true));
        assert_eq!(meta.deck, None);
    }

    #[test]
    fn test_extra_whitespace() {
        let input = "<!--   anki_id :  99 ,  anki_sync :  false   -->";
        let (rest, meta) = parse_flashcard_metadata(input).expect("Should parse");
        assert_eq!(rest, "");
        assert_eq!(meta.id, Some(99));
        assert_eq!(meta.sync, Some(false));
    }

    #[test]
    fn test_trailing_input() {
        let input = "<!-- anki_id: 1 -->some more text";
        let (rest, meta) = parse_flashcard_metadata(input).expect("Should parse");
        assert_eq!(rest, "some more text");
        assert_eq!(meta.id, Some(1));
    }

    #[test]
    fn test_tags_basic() {
        let input = "<!-- anki_tags: [tag1,tag2,tag3] -->";
        let (rest, meta) = parse_flashcard_metadata(input).expect("Should parse");
        assert_eq!(rest, "");
        assert_eq!(meta.tags, Some(vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()]));
    }

    #[test]
    fn test_tags_with_quoted_strings() {
        let input = r#"<!-- anki_tags: [tag1, "tag two", tag3] -->"#;
        let (rest, meta) = parse_flashcard_metadata(input).expect("Should parse");
        assert_eq!(rest, "");
        assert_eq!(meta.tags, Some(vec!["tag1".to_string(), "tag two".to_string(), "tag3".to_string()]));
    }

    #[test]
    fn test_tags_with_extra_whitespace() {
        let input = "<!-- anki_tags: [  tag1 ,  tag2  ,   tag3  ] -->";
        let (rest, meta) = parse_flashcard_metadata(input).expect("Should parse");
        assert_eq!(rest, "");
        assert_eq!(meta.tags, Some(vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()]));
    }

    #[test]
    fn test_tags_with_other_fields() {
        let input = r#"<!-- anki_id: 5, anki_tags: [foo, "bar baz"], anki_sync: true -->"#;
        let (rest, meta) = parse_flashcard_metadata(input).expect("Should parse");
        assert_eq!(rest, "");
        assert_eq!(meta.id, Some(5));
        assert_eq!(meta.sync, Some(true));
        assert_eq!(meta.tags, Some(vec!["foo".to_string(), "bar baz".to_string()]));
    }

    #[test]
    fn test_tags_empty_list_fails() {
        let input = "<!-- anki_tags: [] -->";
        assert!(parse_flashcard_metadata(input).is_err());
    }
}
