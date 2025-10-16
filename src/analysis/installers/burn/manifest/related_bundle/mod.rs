mod action;

use action::Action;
use serde::Deserialize;

/// <https://docs.firegiant.com/wix/schema/wxs/relatedbundle/>
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct RelatedBundle {
    #[serde(rename = "@Code", alias = "@Id")]
    pub code: String,
    #[serde(rename = "@Action")]
    pub action: Action,
}

impl RelatedBundle {
    #[inline]
    pub const fn code(&self) -> &str {
        self.code.as_str()
    }
}
