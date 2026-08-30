use std::str::FromStr;

use thiserror::Error;

pub const EXE: &str = "exe";
pub const MSI: &str = "msi";
pub const MSIX: &str = "msix";
pub const APPX: &str = "appx";
pub const MSIX_BUNDLE: &str = "msixbundle";
pub const APPX_BUNDLE: &str = "appxbundle";
pub const ZIP: &str = "zip";
pub const APP_INSTALLER: &str = "appinstaller";

/// An enumeration of valid file extensions.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FileExtension {
    Exe,
    Msi,
    Msix,
    Appx,
    MsixBundle,
    AppxBundle,
    Zip,
    AppInstaller,
}

impl FileExtension {
    /// Returns `true` if the extension is `.appinstaller`.
    #[must_use]
    #[inline]
    pub const fn is_app_installer(self) -> bool {
        matches!(self, Self::AppInstaller)
    }

    /// Returns the file extension as a static string slice.
    #[must_use]
    #[expect(unused)]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Exe => EXE,
            Self::Msi => MSI,
            Self::Msix => MSIX,
            Self::Appx => APPX,
            Self::MsixBundle => MSIX_BUNDLE,
            Self::AppxBundle => APPX_BUNDLE,
            Self::Zip => ZIP,
            Self::AppInstaller => APP_INSTALLER,
        }
    }
}

#[derive(Debug, Error)]
#[error("{0} is not a supported file extension")]
pub struct UnsupportedFileExtensionError(String);

impl FromStr for FileExtension {
    type Err = UnsupportedFileExtensionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            s if s.eq_ignore_ascii_case(EXE) => Ok(Self::Exe),
            s if s.eq_ignore_ascii_case(MSI) => Ok(Self::Msi),
            s if s.eq_ignore_ascii_case(MSIX) => Ok(Self::Msix),
            s if s.eq_ignore_ascii_case(APPX) => Ok(Self::Appx),
            s if s.eq_ignore_ascii_case(MSIX_BUNDLE) => Ok(Self::MsixBundle),
            s if s.eq_ignore_ascii_case(APPX_BUNDLE) => Ok(Self::AppxBundle),
            s if s.eq_ignore_ascii_case(ZIP) => Ok(Self::Zip),
            s if s.eq_ignore_ascii_case(APP_INSTALLER) => Ok(Self::AppInstaller),
            _ => Err(UnsupportedFileExtensionError(s.to_ascii_lowercase())),
        }
    }
}
