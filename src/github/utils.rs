use std::collections::BTreeSet;
use std::env;
use std::fmt::Write;
use std::num::NonZeroU32;

use clap::{crate_name, crate_version};
use rand::{thread_rng, Rng};
use uuid::Uuid;

use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;
use crate::types::urls::url::Url;
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

pub fn get_pull_request_body(
    issue_resolves: Option<Vec<NonZeroU32>>,
    alternative_text: Option<String>,
    created_with: Option<String>,
    created_with_url: Option<Url>,
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
    if let Some(resolves) = issue_resolves {
        if !resolves.is_empty() {
            let _ = writeln!(body);
            for resolve in BTreeSet::from_iter(resolves) {
                let _ = writeln!(body, "- Resolves #{resolve}");
            }
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
            "manifests/p/Package/Identifier",
            get_package_path(&identifier, None)
        );
    }

    #[test]
    fn test_full_package_path() {
        let identifier = PackageIdentifier::parse("Package.Identifier").unwrap_or_default();
        let version = PackageVersion::new("1.2.3").unwrap_or_default();
        assert_eq!(
            "manifests/p/Package/Identifier/1.2.3",
            get_package_path(&identifier, Some(&version))
        );
    }
}
