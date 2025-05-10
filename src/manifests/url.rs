use std::{
    fmt,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use url::ParseError;
use winget_types::{installer::Architecture, url::DecodedUrl};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Url {
    inner: DecodedUrl,
    override_architecture: Option<Architecture>,
}

impl Url {
    #[inline]
    pub fn override_architecture(&self) -> Option<Architecture> {
        self.override_architecture
    }

    #[inline]
    pub fn inner(&self) -> &DecodedUrl {
        &self.inner
    }

    #[inline]
    pub fn inner_mut(&mut self) -> &mut DecodedUrl {
        &mut self.inner
    }

    #[inline]
    pub fn into_inner(self) -> DecodedUrl {
        self.inner
    }
}

impl Deref for Url {
    type Target = DecodedUrl;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Url {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl fmt::Display for Url {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner().fmt(f)
    }
}

impl FromStr for Url {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (url, architecture) = s.rsplit_once('|').unwrap_or((s, ""));

        Ok(Url {
            inner: url.parse()?,
            override_architecture: architecture.parse().ok(),
        })
    }
}

impl From<DecodedUrl> for Url {
    fn from(url: DecodedUrl) -> Self {
        Self {
            inner: url,
            override_architecture: None,
        }
    }
}
