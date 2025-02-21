use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct NestedInstallerFiles {
    pub relative_file_path: Utf8PathBuf,
    pub portable_command_alias: Option<String>,
}
