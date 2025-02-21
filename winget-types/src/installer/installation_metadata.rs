use std::collections::BTreeSet;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::shared::Sha256String;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct InstallationMetadata {
    pub default_install_location: Option<Utf8PathBuf>,
    pub files: Option<BTreeSet<MetadataFiles>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct MetadataFiles {
    pub relative_file_path: String,
    pub file_sha_256: Option<Sha256String>,
    pub file_type: Option<MetadataFileType>,
    pub invocation_parameter: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum MetadataFileType {
    Launch,
    Uninstall,
    Other,
}
