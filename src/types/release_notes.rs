use nutype::nutype;
use pulldown_cmark::Event::{Code, End, Start, Text};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use std::borrow::Cow;
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

        for event in parser {
            match event {
                Start(tag) => match tag {
                    Tag::CodeBlock(_info) => buffer.push_str("\n\n"),
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
                    Tag::Item => buffer.push_str("- "),
                    _ => (),
                },
                End(tag) => match tag {
                    TagEnd::Table
                    | TagEnd::TableHead
                    | TagEnd::TableRow
                    | TagEnd::Heading(..)
                    | TagEnd::BlockQuote
                    | TagEnd::CodeBlock => buffer.push('\n'),
                    TagEnd::Item => {
                        if &buffer[buffer.len() - 2..] == "- " {
                            buffer.drain(buffer.len() - 2..);
                        } else {
                            buffer.push('\n');
                        }
                    }
                    _ => (),
                },
                Text(text) => {
                    let mut result = String::new();
                    let mut rest = &*text;
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
            ReleaseNotes::new("- No issue link").ok()
        )
    }

    #[test]
    fn test_cut_to_lines() {
        use std::fmt::Write;

        const CHAR_LIMIT: usize = 100;

        let mut buffer = String::new();
        let mut line_count = 1;
        while buffer.chars().count() <= CHAR_LIMIT {
            writeln!(buffer, "Line {line_count}").unwrap();
            line_count += 1;
        }
        let formatted = truncate_with_lines(&buffer, CHAR_LIMIT);
        let formatted_char_count = formatted.chars().count();
        assert!(formatted_char_count < buffer.chars().count());
        assert_eq!(formatted_char_count, formatted.trim().chars().count());
    }
}
