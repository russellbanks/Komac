use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TreeType {
    Blob,
    Tree,
    Commit,
}

impl TreeType {
    /// Returns `true` if the tree type is blob.
    #[inline]
    pub const fn is_blob(self) -> bool {
        matches!(self, Self::Blob)
    }

    /// Returns `true` if the tree type is tree.
    #[inline]
    pub const fn is_tree(self) -> bool {
        matches!(self, Self::Tree)
    }

    /// Returns `true` if the tree type is commit.
    #[inline]
    pub const fn is_commit(self) -> bool {
        matches!(self, Self::Commit)
    }

    /// Returns the tree type as a static string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Blob => "blob",
            Self::Tree => "tree",
            Self::Commit => "commit",
        }
    }
}

impl fmt::Display for TreeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}
