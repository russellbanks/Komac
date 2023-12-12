use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ManifestType {
    #[default]
    Installer,
    DefaultLocale,
    Locale,
    Version,
}
