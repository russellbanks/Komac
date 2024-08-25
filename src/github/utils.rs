use std::collections::BTreeSet;
use std::env;
use std::fmt::Write;
use std::num::NonZeroU32;

use clap::{crate_name, crate_version};
use rand::{thread_rng, Rng};
use uuid::Uuid;

use crate::types::language_tag::LanguageTag;
use crate::types::manifest_type::ManifestType;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;
use crate::types::urls::url::DecodedUrl;
use crate::update_state::UpdateState;

pub fn get_package_path(
    identifier: &PackageIdentifier,
    version: Option<&PackageVersion>,
) -> String {
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
        result.push_str(&version.to_string());
    }
    result
}

pub fn is_manifest_file(
    file_name: &str,
    package_identifier: &PackageIdentifier,
    default_locale: Option<&LanguageTag>,
    manifest_type: &ManifestType,
) -> bool {
    const YAML_EXTENSION: &str = ".yaml";
    const LOCALE_PART: &str = ".locale.";
    const INSTALLER_PART: &str = ".installer";

    let identifier_len = package_identifier.len();
    let file_name_len = file_name.len();

    // All manifest file names start with the package identifier
    if !file_name.starts_with(package_identifier.as_str()) {
        return false;
    }

    // All manifest files end with the YAML extension
    if !file_name.ends_with(YAML_EXTENSION) {
        return false;
    }

    match manifest_type {
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

            locale.map_or(false, |locale| {
                default_locale.is_some_and(|default_locale| match manifest_type {
                    ManifestType::DefaultLocale => default_locale.as_str() == locale,
                    ManifestType::Locale => default_locale.as_str() != locale,
                    _ => false,
                })
            })
        }
    }
}

pub fn get_pull_request_body(
    issue_resolves: Vec<NonZeroU32>,
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
        let mut rng = thread_rng();

        let emoji = if rng.gen_range(0..50) == 0 {
            FRUITS[rng.gen_range(0..FRUITS.len())]
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
        };

        let _ = writeln!(body, " :{emoji}:");
    }
    if !issue_resolves.is_empty() {
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
    use crate::github::utils::get_package_path;
    use crate::types::package_identifier::PackageIdentifier;
    use crate::types::package_version::PackageVersion;

    #[test]
    fn test_partial_package_path() {
        let identifier = PackageIdentifier::parse("Package.Identifier").unwrap_or_default();
        assert_eq!(
            get_package_path(&identifier, None),
            "manifests/p/Package/Identifier"
        );
    }

    #[test]
    fn test_full_package_path() {
        let identifier = PackageIdentifier::parse("Package.Identifier").unwrap_or_default();
        let version = PackageVersion::new("1.2.3").unwrap_or_default();
        assert_eq!(
            get_package_path(&identifier, Some(&version)),
            "manifests/p/Package/Identifier/1.2.3"
        );
    }
}
