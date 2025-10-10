use serde::{Deserialize, Serialize};

use super::{GitFileMode, TreeType};

#[derive(Serialize, Deserialize)]
pub struct TreeObject {
    pub path: String,
    pub mode: GitFileMode,
    r#type: TreeType,
    pub sha: String,
    pub size: Option<i32>,
    pub url: String,
}

impl TreeObject {
    /// Returns `true` if the tree type is blob.
    #[expect(unused)]
    #[inline]
    pub const fn is_blob(&self) -> bool {
        self.r#type.is_blob()
    }

    /// Returns `true` if the tree type is tree.
    #[inline]
    pub const fn is_tree(&self) -> bool {
        self.r#type.is_tree()
    }

    /// Returns `true` if the tree type is commit.
    #[expect(unused)]
    #[inline]
    pub const fn is_commit(&self) -> bool {
        self.r#type.is_commit()
    }
}
