use nutype::nutype;
use percent_encoding::percent_decode_str;

#[nutype(
    sanitize(with = |url: reqwest::Url| {
        reqwest::Url::parse(&percent_decode_str(url.as_str()).decode_utf8().unwrap()).unwrap()
    }),
    derive(Clone, FromStr, Display, Deserialize, Serialize)
)]
pub struct Url(url::Url);
