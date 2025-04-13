use std::{collections::BTreeSet, env, fmt::Write, num::NonZeroU32};

use bon::builder;
use clap::{crate_name, crate_version};
use rand::Rng;
use uuid::Uuid;
use winget_types::{
    LanguageTag, Manifest, ManifestType, ManifestTypeWithLocale, PackageIdentifier, PackageVersion,
    url::DecodedUrl,
};

use crate::update_state::UpdateState;

pub mod pull_request;

const YAML_EXTENSION: &str = ".yaml";
const LOCALE_PART: &str = ".locale.";
const INSTALLER_PART: &str = ".installer";

pub fn get_package_path(
    identifier: &PackageIdentifier,
    version: Option<&PackageVersion>,
    manifest_type: Option<&ManifestTypeWithLocale>,
) -> String {
    let identifier = identifier.as_str();
    let first_character = identifier
        .chars()
        .next()
        .map(|mut first_character| {
            first_character.make_ascii_lowercase();
            first_character
        })
        .unwrap();
    let mut result = format!("manifests/{first_character}");
    for part in identifier.split('.') {
        result.push('/');
        result.push_str(part);
    }
    if let Some(version) = version {
        result.push('/');
        result.push_str(version.as_str());

        // The full manifest file path should only be included if a version was passed in too
        if let Some(manifest_type) = manifest_type {
            result.push('/');
            result.push_str(identifier);
            if matches!(manifest_type, ManifestTypeWithLocale::Installer) {
                result.push_str(INSTALLER_PART);
            } else if let ManifestTypeWithLocale::Locale(tag) = manifest_type {
                result.push_str(LOCALE_PART);
                result.push_str(&tag.to_string());
            }
            result.push_str(YAML_EXTENSION);
        }
    }
    result
}

pub fn is_manifest_file<M: Manifest>(
    file_name: &str,
    package_identifier: &PackageIdentifier,
    default_locale: Option<&LanguageTag>,
) -> bool {
    let package_identifier = package_identifier.as_str();
    let identifier_len = package_identifier.len();
    let file_name_len = file_name.len();

    // All manifest file names start with the package identifier
    if !file_name.starts_with(package_identifier) {
        return false;
    }

    // All manifest files end with the YAML extension
    if !file_name.ends_with(YAML_EXTENSION) {
        return false;
    }

    match M::TYPE {
        ManifestType::Version => file_name_len == identifier_len + YAML_EXTENSION.len(),
        ManifestType::Installer => {
            file_name.get(identifier_len..file_name_len - YAML_EXTENSION.len())
                == Some(INSTALLER_PART)
        }
        ManifestType::DefaultLocale | ManifestType::Locale => {
            // Check if the file name after the identifier starts with `.locale.`
            if file_name.get(identifier_len..identifier_len + LOCALE_PART.len())
                != Some(LOCALE_PART)
            {
                return false;
            }

            let locale = file_name
                .get(identifier_len + LOCALE_PART.len()..file_name_len - YAML_EXTENSION.len());

            locale.is_some_and(|locale| {
                default_locale.is_some_and(|default_locale| match M::TYPE {
                    ManifestType::DefaultLocale => default_locale.to_string() == locale,
                    ManifestType::Locale => default_locale.to_string() != locale,
                    _ => false,
                })
            })
        }
    }
}

#[builder(finish_fn = get)]
pub fn pull_request_body(
    issue_resolves: Option<Vec<NonZeroU32>>,
    alternative_text: Option<String>,
    created_with: Option<String>,
    created_with_url: Option<DecodedUrl>,
) -> String {
    const FRUITS: [&str; 16] = [
        "apple",
        "banana",
        "blueberries",
        "cherries",
        "grapes",
        "green_apple",
        "kiwi_fruit",
        "lemon",
        "mango",
        "melon",
        "peach",
        "pear",
        "pineapple",
        "strawberry",
        "tangerine",
        "watermelon",
    ];

    let mut body = String::new();
    if let Some(alternative_text) = alternative_text {
        let _ = writeln!(body, "### {alternative_text}");
    } else {
        let mut rng = rand::rng();

        let emoji = if rng.random_ratio(1, 50) {
            FRUITS[rng.random_range(0..FRUITS.len())]
        } else {
            "rocket"
        };

        body.push_str("### Pull request has been created with ");

        if let (Some(tool_name), Some(tool_url)) = (created_with, created_with_url) {
            let _ = write!(body, "[{tool_name}]({tool_url})");
        } else {
            let _ = write!(
                body,
                "[{}]({}) v{}",
                crate_name!(),
                env!("CARGO_PKG_REPOSITORY"),
                crate_version!()
            );
        }

        let _ = writeln!(body, " :{emoji}:");
    }
    if let Some(issue_resolves) = issue_resolves.filter(|resolves| !resolves.is_empty()) {
        let _ = writeln!(body);
        for resolve in BTreeSet::from_iter(issue_resolves) {
            let _ = writeln!(body, "- Resolves #{resolve}");
        }
    }
    body
}

pub fn get_branch_name(
    package_identifier: &PackageIdentifier,
    package_version: &PackageVersion,
) -> String {
    /// GitHub rejects branch names longer than 255 bytes. Considering `refs/heads/`, 244 bytes are left for the name.
    const MAX_BRANCH_NAME_LEN: usize = u8::MAX as usize - "refs/heads/".len();
    let mut uuid_buffer = Uuid::encode_buffer();
    let uuid = Uuid::new_v4().simple().encode_upper(&mut uuid_buffer);
    let mut branch_name = format!("{package_identifier}-{package_version}-{uuid}");
    if branch_name.len() > MAX_BRANCH_NAME_LEN {
        branch_name.truncate(MAX_BRANCH_NAME_LEN - uuid.len());
        branch_name.push_str(uuid);
    }
    branch_name
}

pub fn get_commit_title(
    identifier: &PackageIdentifier,
    version: &PackageVersion,
    update_state: &UpdateState,
) -> String {
    format!("{update_state}: {identifier} version {version}")
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use winget_types::{
        LanguageTag, ManifestTypeWithLocale, PackageIdentifier,
        icu_locid::langid,
        installer::InstallerManifest,
        locale::{DefaultLocaleManifest, LocaleManifest},
        version::VersionManifest,
    };

    use crate::github::utils::{get_package_path, is_manifest_file};

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
            get_package_path(&identifier, version.as_ref(), manifest_type.as_ref()),
            expected
        )
    }

    #[test]
    fn valid_installer_manifest_file() {
        assert!(is_manifest_file::<InstallerManifest>(
            "Package.Identifier.installer.yaml",
            &"Package.Identifier".parse::<PackageIdentifier>().unwrap(),
            None,
        ))
    }

    #[test]
    fn invalid_installer_manifest_file() {
        assert!(!is_manifest_file::<InstallerManifest>(
            "Package.Identifier.yaml",
            &"Package.Identifier".parse::<PackageIdentifier>().unwrap(),
            None,
        ))
    }

    #[test]
    fn valid_default_locale_manifest_file() {
        assert!(is_manifest_file::<DefaultLocaleManifest>(
            "Package.Identifier.locale.en-US.yaml",
            &"Package.Identifier".parse::<PackageIdentifier>().unwrap(),
            Some(&LanguageTag::new(langid!("en-US"))),
        ))
    }

    #[test]
    fn invalid_default_locale_manifest_file() {
        assert!(!is_manifest_file::<DefaultLocaleManifest>(
            "Package.Identifier.locale.en-US.yaml",
            &"Package.Identifier".parse::<PackageIdentifier>().unwrap(),
            Some(&LanguageTag::new(langid!("zh-CN"))),
        ))
    }

    #[test]
    fn valid_locale_manifest_file() {
        assert!(is_manifest_file::<LocaleManifest>(
            "Package.Identifier.locale.zh-CN.yaml",
            &"Package.Identifier".parse::<PackageIdentifier>().unwrap(),
            Some(&LanguageTag::new(langid!("en-US"))),
        ))
    }

    #[test]
    fn invalid_locale_manifest_file() {
        assert!(!is_manifest_file::<LocaleManifest>(
            "Package.Identifier.locale.en-US.yaml",
            &"Package.Identifier".parse::<PackageIdentifier>().unwrap(),
            Some(&LanguageTag::new(langid!("en-US"))),
        ))
    }

    #[test]
    fn valid_version_manifest_file() {
        assert!(is_manifest_file::<VersionManifest>(
            "Package.Identifier.yaml",
            &"Package.Identifier".parse::<PackageIdentifier>().unwrap(),
            None,
        ))
    }

    #[test]
    fn invalid_version_manifest_file() {
        assert!(!is_manifest_file::<VersionManifest>(
            "Package.Identifier.installer.yaml",
            &"Package.Identifier".parse::<PackageIdentifier>().unwrap(),
            None,
        ))
    }
}
