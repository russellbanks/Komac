use crate::types::language_tag::LanguageTag;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Deserialize, Serialize)]
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
    Locale(LanguageTag),
    Version,
}
