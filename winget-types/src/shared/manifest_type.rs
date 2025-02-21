use std::fmt::{Display, Formatter};

use icu_locid::LanguageIdentifier;
use serde::{Deserialize, Serialize};

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
            Self::Installer => f.write_str("Installer"),
            Self::DefaultLocale => f.write_str("DefaultLocale"),
            Self::Locale => f.write_str("Locale"),
            Self::Version => f.write_str("Version"),
        }
    }
}

pub enum ManifestTypeWithLocale {
    Installer,
    Locale(LanguageIdentifier),
    Version,
}
