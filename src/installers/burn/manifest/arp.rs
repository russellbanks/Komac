use std::borrow::Cow;

use serde::Deserialize;
use winget_types::Version;

use super::{YesNoButton, bool_from_yes_no};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Arp<'manifest> {
    #[serde(rename = "@Register", deserialize_with = "bool_from_yes_no", default)]
    pub register: bool,
    #[serde(rename = "@DisplayName")]
    pub display_name: &'manifest str,
    #[serde(rename = "@DisplayVersion")]
    pub display_version: Version,
    #[serde(rename = "@InProgressDisplayName")]
    pub in_progress_display_name: Option<&'manifest str>,
    #[serde(rename = "@Publisher")]
    pub publisher: Option<&'manifest str>,
    #[serde(rename = "@HelpLink")]
    pub help_link: Option<Cow<'manifest, str>>,
    #[serde(rename = "@HelpTelephone")]
    pub help_telephone: Option<&'manifest str>,
    #[serde(rename = "@AboutUrl")]
    pub about_url: Option<Cow<'manifest, str>>,
    #[serde(rename = "@UpdateUrl")]
    pub update_url: Option<Cow<'manifest, str>>,
    #[serde(rename = "@ParentDisplayName")]
    pub parent_display_name: Option<&'manifest str>,
    #[serde(rename = "@DisableModify", default)]
    pub disable_modify: YesNoButton,
    #[serde(
        rename = "@DisableRemove",
        deserialize_with = "bool_from_yes_no",
        default
    )]
    pub disable_remove: bool,
}
