use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ManifestType {
    #[default]
    Installer,
    DefaultLocale,
    Locale,
    Version,
}

impl Display for ManifestType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestType::Installer => f.write_str("Installer"),
            ManifestType::DefaultLocale => f.write_str("DefaultLocale"),
            ManifestType::Locale => f.write_str("Locale"),
            ManifestType::Version => f.write_str("Version"),
        }
    }
}
