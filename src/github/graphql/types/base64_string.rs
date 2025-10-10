use base64ct::{Base64, Encoding};

use super::super::github_schema as schema;

/// <https://docs.github.com/graphql/reference/scalars#base64string>
#[derive(cynic::Scalar)]
pub struct Base64String(String);

impl Base64String {
    /// Creates a new `Base64String` from existing Base64.
    pub fn new<S: Into<String>>(base_64: S) -> Self {
        Self(base_64.into())
    }

    /// Encodes input byte slice into a `Base64String` containing Base64.
    ///
    /// # Panics
    ///
    /// If `input` length is greater than `usize::MAX/4`.
    #[inline]
    pub fn encode_string(input: &[u8]) -> Self {
        Self(Base64::encode_string(input))
    }
}

impl<T> From<T> for Base64String
where
    T: Into<String>,
{
    fn from(s: T) -> Self {
        Self::new(s)
    }
}
