mod copyright_url;
mod license_url;
mod package_url;
mod publisher_support_url;
mod publisher_url;
mod release_notes_url;

use std::str::FromStr;

pub use copyright_url::CopyrightUrl;
use derive_more::{Deref, DerefMut, Display};
pub use license_url::LicenseUrl;
pub use package_url::PackageUrl;
use percent_encoding::percent_decode_str;
pub use publisher_support_url::PublisherSupportUrl;
pub use publisher_url::PublisherUrl;
pub use release_notes_url::ReleaseNotesUrl;
use serde::{Deserialize, Serialize};
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
