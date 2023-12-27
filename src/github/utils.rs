use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;
use crate::update_state::UpdateState;
use clap::{crate_name, crate_version};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::env;
use std::time::SystemTime;
use xxhash_rust::xxh3::xxh3_64;

pub fn get_full_package_path(identifier: &PackageIdentifier, version: &PackageVersion) -> String {
    let mut first_character = identifier.chars().next().unwrap();
    first_character.make_ascii_lowercase();
    format!(
        "manifests/{first_character}/{}/{version}",
        identifier.replace('.', "/")
    )
}

pub fn get_package_path(identifier: &PackageIdentifier) -> String {
    let mut first_character = identifier.chars().next().unwrap();
    first_character.make_ascii_lowercase();
    format!(
        "manifests/{first_character}/{}",
        identifier.replace('.', "/")
    )
}

pub fn get_pull_request_body() -> String {
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
    let mut rng = SmallRng::from_entropy();

    let custom_tool_info = if let (Ok(tool_name), Ok(tool_url)) = (
        env::var("KOMAC_CREATED_WITH"),
        env::var("KOMAC_CREATED_WITH_URL"),
    ) {
        format!("[{tool_name}]({tool_url})")
    } else {
        format!(
            "[{}]({}) v{}",
            crate_name!(),
            env!("CARGO_PKG_REPOSITORY"),
            crate_version!()
        )
    };

    let emoji = if rng.gen_range(0..50) == 0 {
        FRUITS[rng.gen_range(0..FRUITS.len())]
    } else {
        "rocket"
    };

    format!("### Pull request has been created with {custom_tool_info} :{emoji}:")
}

pub fn get_branch_name(package_identifier: &str, package_version: &PackageVersion) -> String {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let input = format!("{package_identifier}-{package_version}-{timestamp}");
    let hash = xxh3_64(input.as_bytes());

    format!("{package_identifier}-{package_version}-{hash}")
}

pub fn get_commit_title(
    identifier: &str,
    version: &PackageVersion,
    update_state: &UpdateState,
) -> String {
    format!("{update_state}: {identifier} version {version}")
}
