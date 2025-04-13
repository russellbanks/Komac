use bon::builder;
use color_eyre::Result;
use winget_types::PackageIdentifier;

use crate::manifests::{Manifests, build_manifest_string};

#[builder(finish_fn = create)]
pub fn pr_changes(
    package_identifier: &PackageIdentifier,
    manifests: &Manifests,
    package_path: &str,
    created_with: Option<&str>,
) -> Result<Vec<(String, String)>> {
    let mut path_content_map = vec![
        (
            format!("{package_path}/{package_identifier}.installer.yaml"),
            build_manifest_string(&manifests.installer, created_with)?,
        ),
        (
            format!(
                "{}/{}.locale.{}.yaml",
                package_path, package_identifier, manifests.version.default_locale
            ),
            build_manifest_string(&manifests.default_locale, created_with)?,
        ),
    ];
    for locale_manifest in &manifests.locales {
        path_content_map.push((
            format!(
                "{package_path}/{package_identifier}.locale.{}.yaml",
                locale_manifest.package_locale
            ),
            build_manifest_string(locale_manifest, created_with)?,
        ));
    }
    path_content_map.push((
        format!("{package_path}/{package_identifier}.yaml"),
        build_manifest_string(&manifests.version, created_with)?,
    ));
    Ok(path_content_map)
}
