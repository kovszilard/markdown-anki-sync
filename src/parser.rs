struct Deck {
    name: String,
}

struct Note {
    id: Option<u64>,
    sync: bool,
    front: String,
    back: String,
}

fn get_front_matter(input: &str) -> Option<&str> {
    let trimmed = input.trim_start();
    let rest = trimmed.strip_prefix("---")?;
    let rest = rest
        .strip_prefix('\n')
        .or_else(|| rest.strip_prefix("\r\n"))?;
    let end = rest.find("\n---")?;
    Some(rest[..end].trim_end_matches('\r'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_get_front_matter() {
        let input = indoc! {"
            ---
            key: value
            ---
            # Body
        "};
        assert_eq!(get_front_matter(input), Some("key: value"));
    }

    #[test]
    fn test_get_front_matter_no_frontmatter() {
        let input = indoc! {"
            # Just a heading
            Some text
        "};
        assert_eq!(get_front_matter(input), None);
    }

    #[test]
    fn test_get_front_matter_multiple_lines() {
        let input = indoc! {"
            ---
            key1: value1
            key2: value2
            ---
            # Body
        "};
        assert_eq!(get_front_matter(input), Some("key1: value1\nkey2: value2"));
    }
}
