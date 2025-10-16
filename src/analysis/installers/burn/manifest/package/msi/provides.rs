use serde::Deserialize;

use crate::analysis::installers::burn::manifest::yes_no::YesNoButton;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Provides {
    #[serde(rename = "@Key")]
    pub key: String,
    #[serde(rename = "@Version")]
    pub version: Option<String>,
    #[serde(rename = "@DisplayName")]
    display_name: Option<String>,
    #[serde(rename = "@Imported", default)]
    pub imported: YesNoButton,
}

impl Provides {
    #[inline]
    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }
}
