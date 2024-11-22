use crate::github::github_client::Manifests;
use crate::manifests::build_manifest_string;
use crate::types::package_identifier::PackageIdentifier;
use bon::builder;
use color_eyre::Result;

#[builder(finish_fn = create)]
pub fn pr_changes(
    package_identifier: &PackageIdentifier,
    manifests: &Manifests,
    package_path: &str,
    created_with: &Option<String>,
) -> Result<Vec<(String, String)>> {
    let mut path_content_map = vec![
        (
            format!("{package_path}/{package_identifier}.installer.yaml"),
            build_manifest_string(&manifests.installer_manifest, created_with)?,
        ),
        (
            format!(
                "{}/{}.locale.{}.yaml",
                package_path, package_identifier, manifests.version_manifest.default_locale
            ),
            build_manifest_string(&manifests.default_locale_manifest, created_with)?,
        ),
    ];
    for locale_manifest in &manifests.locale_manifests {
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
        build_manifest_string(&manifests.version_manifest, created_with)?,
    ));
    Ok(path_content_map)
}
