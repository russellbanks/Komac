use nutype::nutype;
use percent_encoding::percent_decode_str;
use url::Url;

#[nutype(
    sanitize(with = |url: Url| {
        Url::parse(&percent_decode_str(url.as_str()).decode_utf8().unwrap()).unwrap()
    }),
    default = Url::parse("https://www.example.com").unwrap(),
    derive(Clone, Deref, FromStr, Display, Default, Deserialize, Serialize, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)
)]
pub struct DecodedUrl(Url);

impl From<Url> for DecodedUrl {
    fn from(value: Url) -> Self {
        Self::new(value)
    }
}
