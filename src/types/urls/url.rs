use derive_more::{Deref, DerefMut, Display};
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use url::{ParseError, Url};

#[derive(
    Clone,
    Debug,
    Display,
    Deref,
    DerefMut,
    Hash,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
)]
pub struct DecodedUrl(Url);

impl FromStr for DecodedUrl {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Url::parse(&percent_decode_str(s).decode_utf8_lossy()).map(DecodedUrl)
    }
}

impl Default for DecodedUrl {
    fn default() -> Self {
        Self(Url::parse("https://example.com").unwrap())
    }
}
