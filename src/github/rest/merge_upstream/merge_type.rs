use std::fmt;

use serde::Deserialize;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MergeType {
    Merge,
    FastForward,
    None,
}

impl MergeType {
    /// Returns the merge type as a kebab-case static string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Merge => "merge",
            Self::FastForward => "fast-forward",
            Self::None => "none",
        }
    }
}

impl fmt::Display for MergeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}
