use icu_locid::LanguageIdentifier;

#[expect(dead_code)]
pub enum ManifestTypeWithLocale {
    Installer,
    Locale(LanguageIdentifier),
    Version,
}
