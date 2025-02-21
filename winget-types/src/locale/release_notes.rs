use std::{borrow::Cow, str::FromStr};

use derive_more::{Deref, Display};
use serde::Serialize;
use serde_with::DeserializeFromStr;
use thiserror::Error;

#[derive(
    Clone,
    Debug,
    Default,
    Deref,
    Display,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Serialize,
    DeserializeFromStr,
)]
pub struct ReleaseNotes(String);

#[derive(Error, Debug, Eq, PartialEq)]
pub enum ReleaseNotesError {
    #[error("Release notes cannot be empty")]
    Empty,
}

impl ReleaseNotes {
    pub const MAX_CHAR_LENGTH: usize = 10000;

    pub fn new<S: AsRef<str>>(value: S) -> Result<Self, ReleaseNotesError> {
        let result = truncate_with_lines::<{ Self::MAX_CHAR_LENGTH }>(value.as_ref().trim());
        if result.is_empty() {
            Err(ReleaseNotesError::Empty)
        } else {
            Ok(Self(result.into_owned()))
        }
    }
}

impl FromStr for ReleaseNotes {
    type Err = ReleaseNotesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

fn truncate_with_lines<const N: usize>(value: &str) -> Cow<str> {
    if value.chars().count() <= N {
        return Cow::Borrowed(value);
    }

    let mut result = String::with_capacity(N);
    let mut current_size = 0;

    for (iter_count, line) in value.lines().enumerate() {
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
    use crate::locale::release_notes::truncate_with_lines;

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
