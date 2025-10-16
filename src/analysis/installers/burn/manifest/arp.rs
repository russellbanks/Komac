use serde::Deserialize;
use winget_types::Version;

use super::{YesNoButton, bool_from_yes_no};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Arp {
    #[serde(rename = "@Register", deserialize_with = "bool_from_yes_no", default)]
    pub register: bool,
    #[serde(rename = "@DisplayName")]
    display_name: String,
    #[serde(rename = "@DisplayVersion")]
    display_version: Version,
    #[serde(rename = "@InProgressDisplayName")]
    pub in_progress_display_name: Option<String>,
    #[serde(rename = "@Publisher")]
    publisher: Option<String>,
    #[serde(rename = "@HelpLink")]
    pub help_link: Option<String>,
    #[serde(rename = "@HelpTelephone")]
    pub help_telephone: Option<String>,
    #[serde(rename = "@AboutUrl")]
    pub about_url: Option<String>,
    #[serde(rename = "@UpdateUrl")]
    pub update_url: Option<String>,
    #[serde(rename = "@ParentDisplayName")]
    pub parent_display_name: Option<String>,
    #[serde(rename = "@DisableModify", default)]
    pub disable_modify: YesNoButton,
    #[serde(
        rename = "@DisableRemove",
        deserialize_with = "bool_from_yes_no",
        default
    )]
    pub disable_remove: bool,
}

impl Arp {
    #[inline]
    pub const fn display_name(&self) -> &str {
        self.display_name.as_str()
    }

    #[inline]
    pub fn publisher(&self) -> Option<&str> {
        self.publisher.as_deref()
    }

    #[inline]
    pub const fn display_version(&self) -> &Version {
        &self.display_version
    }
}
