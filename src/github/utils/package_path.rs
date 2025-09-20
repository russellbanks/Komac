use std::fmt::{Display, Formatter, Write};

use winget_types::{ManifestTypeWithLocale, PackageIdentifier, PackageVersion};

use super::{INSTALLER_PART, LOCALE_PART, YAML_EXTENSION};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct PackagePath(String);

impl PackagePath {
    pub fn new(
        identifier: &PackageIdentifier,
        version: Option<&PackageVersion>,
        manifest_type: Option<&ManifestTypeWithLocale>,
    ) -> Self {
        let first_character = identifier.as_str().chars().next().map_or_else(
            || unreachable!("Package identifiers cannot be empty"),
            |mut first| {
                first.make_ascii_lowercase();
                first
            },
        );

        // manifests/p
        let mut result = format!("manifests/{first_character}");

        // manifests/p/Package/Identifier
        for part in identifier.as_str().split('.') {
            let _ = write!(result, "/{part}");
        }

        // manifests/p/Package/Identifier/1.2.3
        if let Some(version) = version {
            let _ = write!(result, "/{version}");

            // The full manifest file path should only be included if a version was passed in too
            if let Some(manifest_type) = manifest_type {
                let _ = write!(result, "/{identifier}");
                if matches!(manifest_type, ManifestTypeWithLocale::Installer) {
                    // manifests/p/Package/Identifier/1.2.3/Package.Identifier.installer.yaml
                    result.push_str(INSTALLER_PART);
                } else if let ManifestTypeWithLocale::Locale(tag) = manifest_type {
                    let _ = write!(result, "{LOCALE_PART}{tag}");
                }
                result.push_str(YAML_EXTENSION);
            }
        }

        Self(result)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Display for PackagePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use winget_types::{ManifestTypeWithLocale, PackageIdentifier, icu_locale::langid};

    use super::PackagePath;

    #[rstest]
    #[case("Package.Identifier", None, None, "manifests/p/Package/Identifier")]
    #[case(
        "Package.Identifier",
        Some("1.2.3"),
        None,
        "manifests/p/Package/Identifier/1.2.3"
    )]
    #[case(
        "Package.Identifier",
        Some("1.2.3"),
        Some(ManifestTypeWithLocale::Installer),
        "manifests/p/Package/Identifier/1.2.3/Package.Identifier.installer.yaml"
    )]
    #[case(
        "Package.Identifier",
        Some("1.2.3"),
        Some(ManifestTypeWithLocale::Locale(langid!("en-US"))),
        "manifests/p/Package/Identifier/1.2.3/Package.Identifier.locale.en-US.yaml"
    )]
    #[case(
        "Package.Identifier",
        Some("1.2.3"),
        Some(ManifestTypeWithLocale::Locale(langid!("zh-CN"))),
        "manifests/p/Package/Identifier/1.2.3/Package.Identifier.locale.zh-CN.yaml"
    )]
    #[case(
        "Package.Identifier",
        Some("1.2.3"),
        Some(ManifestTypeWithLocale::Version),
        "manifests/p/Package/Identifier/1.2.3/Package.Identifier.yaml"
    )]
    fn package_paths(
        #[case] identifier: &str,
        #[case] version: Option<&str>,
        #[case] manifest_type: Option<ManifestTypeWithLocale>,
        #[case] expected: &str,
    ) {
        let identifier = identifier.parse::<PackageIdentifier>().unwrap();
        let version = version.and_then(|version| version.parse().ok());
        assert_eq!(
            PackagePath::new(&identifier, version.as_ref(), manifest_type.as_ref()).as_str(),
            expected
        )
    }
}
