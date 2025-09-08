use std::fmt;

use serde::Deserialize;

/// <https://github.com/wixtoolset/wix/blob/main/src/api/wix/WixToolset.Data/PackagingType.cs#L5>
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Packaging {
    #[default]
    Unknown,
    Embedded,
    External,
}

impl Packaging {
    /// Returns the packaging as a static string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Embedded => "Embedded",
            Self::External => "External",
        }
    }
}

impl fmt::Display for Packaging {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}
