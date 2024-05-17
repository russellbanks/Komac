use nutype::nutype;
use pulldown_cmark::Event::{Code, End, Start, Text};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use std::borrow::Cow;
use std::collections::HashMap;
use std::num::NonZeroU32;

#[nutype(
    sanitize(with = |input| truncate_with_lines(&input, 10000).into_owned(), trim),
    validate(len_char_min = 1, len_char_max = 10000),
    default = "Release Notes",
    derive(Clone, Default, FromStr, Display, Deserialize, Serialize, PartialEq, Eq, Debug)
)]
pub struct ReleaseNotes(String);

impl ReleaseNotes {
    pub fn format(body: &str, owner: &str, repo: &str) -> Option<Self> {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);

        let parser = Parser::new_ext(body, options);
        let mut buffer = String::new();

        let mut ordered_list_map = HashMap::new();
        let mut list_item_level = 0;
        for event in parser {
            match event {
                Start(tag) => match tag {
                    Tag::BlockQuote(_) | Tag::CodeBlock(_) => {
                        if !buffer.ends_with('\n') {
                            buffer.push('\n');
                        }
                    }
                    Tag::Link {
                        link_type: _,
                        dest_url: _,
                        title,
                        id: _,
                    }
                    | Tag::Image {
                        link_type: _,
                        dest_url: _,
                        title,
                        id: _,
                    } => {
                        if !title.is_empty() {
                            buffer.push_str(&title);
                        }
                    }
                    Tag::List(first_index) => {
                        if let Some(index) = first_index {
                            ordered_list_map.insert(list_item_level, index);
                        }
                        if !buffer.ends_with('\n') {
                            buffer.push('\n');
                        }
                    }
                    Tag::Item => {
                        for _ in 0..list_item_level {
                            buffer.push_str("    ");
                        }
                        if let Some(index) = ordered_list_map.get_mut(&list_item_level) {
                            buffer.push_str(&format!("{index}. "));
                            *index += 1;
                        } else {
                            buffer.push_str("- ");
                        }
                        list_item_level += 1;
                    }
                    _ => (),
                },
                End(tag) => match tag {
                    TagEnd::Heading(_)
                    | TagEnd::BlockQuote
                    | TagEnd::CodeBlock
                    | TagEnd::Table
                    | TagEnd::TableHead
                    | TagEnd::TableRow => buffer.push('\n'),
                    TagEnd::List(_) => {
                        ordered_list_map.remove(&list_item_level);
                        if list_item_level >= 1 && buffer.ends_with('\n') {
                            buffer.pop();
                        }
                    }
                    TagEnd::Item => {
                        let second_last_char_pos = buffer
                            .char_indices()
                            .nth_back(1)
                            .map_or(buffer.len(), |(pos, _)| pos);
                        if &buffer[second_last_char_pos..] == "- " {
                            buffer.drain(second_last_char_pos..);
                        } else {
                            buffer.push('\n');
                        }
                        list_item_level -= 1;
                    }
                    _ => (),
                },
                Text(text) => {
                    let mut result = String::new();
                    let mut rest = &*remove_sha1(&text);
                    let prefix = "https://github.com/";

                    while let Some(start) = rest.find(prefix) {
                        result.push_str(&rest[..start]);
                        rest = &rest[start..];

                        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
                        let url = &rest[..end];
                        let mut parts = url.trim_start_matches(prefix).split('/');

                        if let (
                            Some(repo_owner),
                            Some(repo_name),
                            Some(issue_type),
                            Some(issue_number),
                        ) = (parts.next(), parts.next(), parts.next(), parts.next())
                        {
                            if (issue_type == "pull" || issue_type == "issues")
                                && issue_number.parse::<NonZeroU32>().is_ok()
                            {
                                if repo_owner != owner || repo_name != repo {
                                    result.push_str(repo_owner);
                                    result.push('/');
                                    result.push_str(repo_name);
                                }
                                result.push('#');
                                result.push_str(issue_number);
                            } else if issue_type == "compare" || issue_type == "releases" {
                                result.push_str(url);
                            }
                        }

                        rest = &rest[end..];
                    }
                    result.push_str(rest);
                    buffer.push_str(&result);
                }
                Code(code) => buffer.push_str(&code),
                Event::SoftBreak | Event::HardBreak | Event::Rule => buffer.push('\n'),
                _ => (),
            }
        }
        Self::new(buffer).ok()
    }
}

fn remove_sha1(input: &str) -> String {
    const SHA1_LEN: usize = 40;
    let mut result = String::new();
    let mut buffer = heapless::String::<SHA1_LEN>::new();

    for character in input.chars() {
        if character.is_ascii_hexdigit() && buffer.len() < SHA1_LEN {
            buffer.push(character).unwrap();
        } else if !character.is_ascii_hexdigit() && buffer.len() == SHA1_LEN {
            buffer.clear();
        } else {
            result.push_str(&buffer);
            buffer.clear();
            result.push(character);
        }
    }

    if buffer.len() != SHA1_LEN {
        result.push_str(&buffer);
    }

    result
}

fn truncate_with_lines(input: &str, char_limit: usize) -> Cow<str> {
    if input.chars().count() <= char_limit {
        return Cow::Borrowed(input);
    }

    let mut result = String::with_capacity(char_limit);
    let mut current_size = 0;

    for (iter_count, line) in input.lines().enumerate() {
        let prospective_size = current_size + line.chars().count() + 1; // +1 for NewLine
        if prospective_size > char_limit {
            break;
        }
        if iter_count != 0 {
            result.push('\n');
        }
        result.push_str(line);
        current_size = prospective_size;
    }

    Cow::Owned(result)
}

#[cfg(test)]
mod tests {
    use crate::types::release_notes::{truncate_with_lines, ReleaseNotes};
    use indoc::indoc;

    #[test]
    fn test_issue_formatting() {
        let value = "- Issue https://github.com/owner/repo/issues/123";
        assert_eq!(
            ReleaseNotes::format(value, "owner", "repo"),
            ReleaseNotes::new("- Issue #123").ok()
        )
    }

    #[test]
    fn test_different_repo_issue_formatting() {
        let value = "- Issue https://github.com/different/repo/issues/123";
        assert_eq!(
            ReleaseNotes::format(value, "owner", "repo"),
            ReleaseNotes::new("- Issue different/repo#123").ok()
        )
    }

    #[test]
    fn test_multiple_issues_formatting() {
        let value = "- Issue https://github.com/owner/repo/issues/123 and https://github.com/owner/repo/issues/321";
        assert_eq!(
            ReleaseNotes::format(value, "owner", "repo"),
            ReleaseNotes::new("- Issue #123 and #321").ok()
        )
    }

    #[test]
    fn test_no_urls() {
        let value = "- No issue link";
        assert_eq!(
            ReleaseNotes::format(value, "owner", "repo"),
            ReleaseNotes::new(value).ok()
        )
    }

    #[test]
    fn test_full_changelog_url() {
        let value = "Full Changelog: https://github.com/owner/repo/compare/v1.0.0...v1.1.0";
        assert_eq!(
            ReleaseNotes::format(value, "owner", "repo"),
            ReleaseNotes::new(value).ok()
        )
    }

    #[test]
    fn test_release_url() {
        let value = "Previous release: https://github.com/owner/repo/releases/tag/1.2.3";
        assert_eq!(
            ReleaseNotes::format(value, "owner", "repo"),
            ReleaseNotes::new(value).ok()
        )
    }

    #[test]
    fn test_header_syntax_removed() {
        let value = indoc! {"
        # Header 1
        ## Header 2
        ### Header 3
        "};
        let expected = indoc! {"
        Header 1
        Header 2
        Header 3
        "};
        assert_eq!(
            ReleaseNotes::format(value, "owner", "repo"),
            ReleaseNotes::new(expected).ok()
        )
    }

    #[test]
    fn test_strikethrough_removed() {
        assert_eq!(
            ReleaseNotes::format("~~Strikethrough text~~", "owner", "repo"),
            ReleaseNotes::new("Strikethrough text").ok()
        )
    }

    #[test]
    fn test_bold_removed() {
        assert_eq!(
            ReleaseNotes::format("**Bold text**", "owner", "repo"),
            ReleaseNotes::new("Bold text").ok()
        )
    }

    #[test]
    fn test_asterisk_bullet_points() {
        let value = indoc! {"
        * Bullet point 1
        * Bullet point 2
        "};
        let expected = indoc! {"
        - Bullet point 1
        - Bullet point 2
        "};
        assert_eq!(
            ReleaseNotes::format(value, "owner", "repo"),
            ReleaseNotes::new(expected).ok()
        )
    }

    #[test]
    fn test_ordered_list() {
        let value = indoc! {"
        1. Item number 1
            1. Item number 1.1
            2. Item number 1.2
                1. Item number 1.2.1
                2. Item number 1.2.2
                3. Item number 1.2.3
        2. Item number 2
            1. Item number 2.1
            2. Item number 2.2
        "};
        assert_eq!(
            ReleaseNotes::format(value, "owner", "repo"),
            ReleaseNotes::new(value).ok()
        )
    }

    #[test]
    fn test_nested_list_items() {
        let value = indoc! {"
        - Bullet point 1
            - 2nd level nested bullet point 1
            - 2nd level nested bullet point 2
                - 3rd level nested bullet point 1
                - 3rd level nested bullet point 2
                    - 4th level nested bullet point 1
                    - 4th level nested bullet point 2
        - Bullet point 2
        "};
        assert_eq!(
            ReleaseNotes::format(value, "owner", "repo"),
            ReleaseNotes::new(value).ok()
        )
    }

    #[test]
    fn test_sha1_removed() {
        use rand::random;
        use sha1::{Digest, Sha1};

        let random_hash = base16ct::lower::encode_string(&Sha1::digest(random::<[u8; 1 << 4]>()));
        let value = format!("- {random_hash} Bullet point 1 {random_hash}");
        assert_eq!(
            ReleaseNotes::format(&value, "owner", "repo"),
            ReleaseNotes::new("- Bullet point 1").ok()
        )
    }

    #[test]
    fn test_truncate_to_lines() {
        use std::fmt::Write;

        const CHAR_LIMIT: usize = 100;

        let mut buffer = String::new();
        let mut line_count = 0;
        while buffer.chars().count() <= CHAR_LIMIT {
            line_count += 1;
            writeln!(buffer, "Line {line_count}").unwrap();
        }
        let formatted = truncate_with_lines(&buffer, CHAR_LIMIT);
        let formatted_char_count = formatted.chars().count();
        assert!(formatted_char_count < buffer.chars().count());
        assert_eq!(formatted_char_count, formatted.trim().chars().count());
    }
}
