use nutype::nutype;
use percent_encoding::percent_decode_str;

#[nutype(
    sanitize(with = |url: url::Url| {
        url::Url::parse(&percent_decode_str(url.as_str()).decode_utf8().unwrap()).unwrap()
    }),
    default = url::Url::parse("https://www.example.com").unwrap(),
    derive(Clone, Deref, FromStr, Display, Default, Deserialize, Serialize, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)
)]
pub struct Url(url::Url);

impl From<url::Url> for Url {
    fn from(value: url::Url) -> Self {
        Self::new(value)
    }
}
