use derive_more::Display;
use icu_locid::LanguageIdentifier;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Display, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ManifestType {
    #[default]
    Installer,
    DefaultLocale,
    Locale,
    Version,
}

#[expect(dead_code)]
pub enum ManifestTypeWithLocale {
    Installer,
    Locale(LanguageIdentifier),
    Version,
}
