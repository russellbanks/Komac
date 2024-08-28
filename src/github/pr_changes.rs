use color_eyre::Result;
use derive_builder::Builder;

use crate::github::github_client::Manifests;
use crate::manifests::build_manifest_string;
use crate::types::package_identifier::PackageIdentifier;

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct PRChanges<'a> {
    package_identifier: &'a PackageIdentifier,
    manifests: Manifests,
    package_path: &'a str,
    created_with: &'a Option<String>,
}

impl PRChanges<'_> {
    pub fn create(&self) -> Result<Vec<(String, String)>> {
        let mut path_content_map = vec![
            (
                format!(
                    "{}/{}.installer.yaml",
                    self.package_path, self.package_identifier
                ),
                build_manifest_string(&self.manifests.installer_manifest, self.created_with)?,
            ),
            (
                format!(
                    "{}/{}.locale.{}.yaml",
                    self.package_path,
                    self.package_identifier,
                    self.manifests.version_manifest.default_locale
                ),
                build_manifest_string(&self.manifests.default_locale_manifest, self.created_with)?,
            ),
        ];
        for locale_manifest in &self.manifests.locale_manifests {
            path_content_map.push((
                format!(
                    "{}/{}.locale.{}.yaml",
                    self.package_path, self.package_identifier, locale_manifest.package_locale
                ),
                build_manifest_string(locale_manifest, self.created_with)?,
            ));
        }
        path_content_map.push((
            format!("{}/{}.yaml", self.package_path, self.package_identifier),
            build_manifest_string(&self.manifests.version_manifest, self.created_with)?,
        ));
        Ok(path_content_map)
    }
}
