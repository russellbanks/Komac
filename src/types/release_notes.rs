use derive_more::Constructor;
use html2text::render::{TaggedLine, TextDecorator};
use nutype::nutype;
use std::borrow::Cow;

#[derive(Constructor)]
struct GitHubHtmlDecorator;

impl TextDecorator for GitHubHtmlDecorator {
    type Annotation = ();

    fn decorate_link_start(&mut self, _url: &str) -> (String, Self::Annotation) {
        (String::new(), ())
    }

    fn decorate_link_end(&mut self) -> String {
        String::new()
    }

    fn decorate_em_start(&self) -> (String, Self::Annotation) {
        (String::new(), ())
    }

    fn decorate_em_end(&self) -> String {
        String::new()
    }

    fn decorate_strong_start(&self) -> (String, Self::Annotation) {
        (String::new(), ())
    }

    fn decorate_strong_end(&self) -> String {
        String::new()
    }

    fn decorate_strikeout_start(&self) -> (String, Self::Annotation) {
        (String::new(), ())
    }

    fn decorate_strikeout_end(&self) -> String {
        String::new()
    }

    fn decorate_code_start(&self) -> (String, Self::Annotation) {
        (String::new(), ())
    }

    fn decorate_code_end(&self) -> String {
        String::new()
    }

    fn decorate_preformat_first(&self) -> Self::Annotation {}

    fn decorate_preformat_cont(&self) -> Self::Annotation {}

    fn decorate_image(&mut self, _src: &str, title: &str) -> (String, Self::Annotation) {
        (title.to_string(), ())
    }

    fn header_prefix(&self, _level: usize) -> String {
        String::new()
    }

    fn quote_prefix(&self) -> String {
        String::from("> ")
    }

    fn unordered_item_prefix(&self) -> String {
        String::from("- ")
    }

    fn ordered_item_prefix(&self, i: i64) -> String {
        format!("{i}. ")
    }

    fn make_subblock_decorator(&self) -> Self {
        Self::new()
    }

    fn finalise(&mut self, _links: Vec<String>) -> Vec<TaggedLine<()>> {
        Vec::new()
    }
}

#[nutype(
    sanitize(with = |input| truncate_with_lines::<10000>(&input).into_owned(), trim),
    validate(len_char_min = 1, len_char_max = 10000),
    default = "Release Notes",
    derive(Clone, Default, FromStr, Display, Deserialize, Serialize, PartialEq, Eq, Debug)
)]
pub struct ReleaseNotes(String);

impl ReleaseNotes {
    pub fn format(body: &str) -> Option<Self> {
        html2text::from_read_with_decorator(body.as_bytes(), usize::MAX, GitHubHtmlDecorator::new())
            .ok()
            .and_then(|text| Self::try_new(text.replace("\n\n", "\n")).ok())
    }
}

fn truncate_with_lines<const N: usize>(input: &str) -> Cow<str> {
    if input.chars().count() <= N {
        return Cow::Borrowed(input);
    }

    let mut result = String::with_capacity(N);
    let mut current_size = 0;

    for (iter_count, line) in input.lines().enumerate() {
        let prospective_size = current_size + line.chars().count() + "\n".len();
        if prospective_size > N {
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
    use crate::types::release_notes::truncate_with_lines;

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
        let formatted = truncate_with_lines::<CHAR_LIMIT>(&buffer);
        let formatted_char_count = formatted.chars().count();
        assert!(formatted_char_count < buffer.chars().count());
        assert_eq!(formatted.trim().chars().count(), formatted_char_count);
    }
}
