use crate::manifests::default_locale_manifest::DefaultLocaleManifest;
use crate::manifests::installer_manifest::InstallerManifest;
use crate::manifests::locale_manifest::LocaleManifest;
use crate::manifests::version_manifest::VersionManifest;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;

pub enum Manifest {
    Installer(InstallerManifest),
    DefaultLocale(DefaultLocaleManifest),
    Locale(LocaleManifest),
    Version(VersionManifest),
}

impl Manifest {
    pub const fn package_identifier(&self) -> &PackageIdentifier {
        match self {
            Self::Installer(installer) => &installer.package_identifier,
            Self::DefaultLocale(default_locale) => &default_locale.package_identifier,
            Self::Locale(locale) => &locale.package_identifier,
            Self::Version(version) => &version.package_identifier,
        }
    }

    pub const fn package_version(&self) -> &PackageVersion {
        match self {
            Self::Installer(installer) => &installer.package_version,
            Self::DefaultLocale(default_locale) => &default_locale.package_version,
            Self::Locale(locale) => &locale.package_version,
            Self::Version(version) => &version.package_version,
        }
    }
}
