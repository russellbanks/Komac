use crate::installers::inno::header::flags::PrivilegesRequiredOverrides;
use crate::manifests::installer_manifest::ElevationRequirement;
use strum::FromRepr;
// This file defines enums corresponding to Inno Setup's header values. Each enum is represented as
// a u8 as Inno Setup stores these values in a single byte. For example, 0 = Classic, 1 = Modern.

/// <https://jrsoftware.org/ishelp/index.php?topic=setup_wizardstyle>
#[derive(Debug, Default, FromRepr)]
#[repr(u8)]
pub enum InnoStyle {
    #[default]
    Classic,
    Modern,
}

/// <https://jrsoftware.org/ishelp/index.php?topic=setup_wizardimagealphaformat>
#[derive(Debug, Default, FromRepr)]
#[repr(u8)]
pub enum ImageAlphaFormat {
    #[default]
    Ignored,
    Defined,
    Premultiplied,
}

#[derive(Debug, Default, FromRepr)]
#[repr(u8)]
pub enum InstallVerbosity {
    #[default]
    Normal,
    Silent,
    VerySilent,
}

#[derive(Debug, Default, FromRepr)]
#[repr(u8)]
pub enum LogMode {
    Append,
    #[default]
    New,
    Overwrite,
}

#[derive(Debug, Default, FromRepr)]
#[repr(u8)]
pub enum AutoBool {
    #[default]
    Auto,
    No,
    Yes,
}

#[derive(Debug, Default, PartialEq, Eq, FromRepr)]
#[repr(u8)]
pub enum PrivilegeLevel {
    #[default]
    None,
    PowerUser,
    Admin,
    Lowest,
}

impl PrivilegeLevel {
    pub const fn to_elevation_requirement(
        &self,
        overrides: &PrivilegesRequiredOverrides,
    ) -> Option<ElevationRequirement> {
        match self {
            Self::Admin | Self::PowerUser => Some(ElevationRequirement::ElevatesSelf),
            _ if !overrides.is_empty() => Some(ElevationRequirement::ElevatesSelf),
            _ => None,
        }
    }
}

/// <https://jrsoftware.org/ishelp/index.php?topic=setup_languagedetectionmethod>
#[derive(Debug, Default, FromRepr)]
#[repr(u8)]
pub enum LanguageDetection {
    #[default]
    UILanguage,
    LocaleLanguage,
    NoLanguageDetection,
}

#[derive(Debug, Default, FromRepr)]
#[repr(u8)]
pub enum Compression {
    Stored,
    Zlib,
    BZip2,
    LZMA1,
    LZMA2,
    #[default]
    Unknown = u8::MAX, // Set to u8::MAX to avoid conflicts with future variants
}
