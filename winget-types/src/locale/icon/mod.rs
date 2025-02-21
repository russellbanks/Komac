pub mod file_type;
pub mod resolution;
pub mod theme;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

use crate::{
    locale::icon::{file_type::IconFileType, resolution::IconResolution, theme::IconTheme},
    shared::Sha256String,
};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Icon {
    #[serde(rename = "IconUrl")]
    pub url: Url,
    #[serde(rename = "IconFileType")]
    pub file_type: Option<IconFileType>,
    #[serde(rename = "IconResolution")]
    pub resolution: Option<IconResolution>,
    #[serde(rename = "IconTheme")]
    pub theme: Option<IconTheme>,
    #[serde(rename = "IconSha256")]
    pub sha_256: Option<Sha256String>,
}
