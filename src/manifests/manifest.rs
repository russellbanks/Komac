use winget_types::{
    DefaultLocaleManifest, InstallerManifest, LocaleManifest, Manifest as WingetManifest,
    PackageIdentifier, PackageVersion, VersionManifest,
};

pub enum Manifest {
    Installer(InstallerManifest),
    DefaultLocale(DefaultLocaleManifest),
    Locale(LocaleManifest),
    Version(VersionManifest),
}

impl Manifest {
    /// Returns the package identifier.
    pub fn package_identifier(&self) -> &PackageIdentifier {
        match self {
            Self::Installer(installer) => installer.package_identifier(),
            Self::DefaultLocale(default_locale) => default_locale.package_identifier(),
            Self::Locale(locale) => locale.package_identifier(),
            Self::Version(version) => version.package_identifier(),
        }
    }

    /// Returns the package version.
    pub fn package_version(&self) -> &PackageVersion {
        match self {
            Self::Installer(installer) => installer.package_version(),
            Self::DefaultLocale(default_locale) => default_locale.package_version(),
            Self::Locale(locale) => locale.package_version(),
            Self::Version(version) => version.package_version(),
        }
    }
}
