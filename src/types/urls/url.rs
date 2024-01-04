use nutype::nutype;
use percent_encoding::percent_decode_str;
use std::str::FromStr;

#[nutype(
    sanitize(with = |url: url::Url| {
        url::Url::parse(&percent_decode_str(url.as_str()).decode_utf8().unwrap()).unwrap()
    }),
    derive(Clone, Deref, FromStr, Display, Deserialize, Serialize, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)
)]
pub struct Url(url::Url);

impl Url {
    pub fn parse(url: &str) -> Result<Self, UrlParseError> {
        Self::from_str(url)
    }
}
