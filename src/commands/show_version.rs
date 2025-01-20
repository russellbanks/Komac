use clap::Parser;
use color_eyre::Result;

use crate::credential::handle_token;
use crate::github::github_client::GitHub;
use crate::manifests::print_changes;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;

/// Output the manifests for a given package and version
#[expect(clippy::struct_excessive_bools)]
#[derive(Parser)]
pub struct ShowVersion {
    /// The package's unique identifier
    #[arg()]
    package_identifier: PackageIdentifier,

    /// The package's version
    #[arg(short = 'v', long = "version")]
    package_version: Option<PackageVersion>,

    /// Switch to display the installer manifest
    #[arg(short, long)]
    installer_manifest: bool,

    /// Switch to display the default locale manifest
    #[arg(short, long = "defaultlocale-manifest")]
    default_locale_manifest: bool,

    /// Switch to display all locale manifests
    #[arg(short, long)]
    locale_manifests: bool,

    /// Switch to display the version manifest
    #[arg(long)]
    version_manifest: bool,

    /// GitHub personal access token with the `public_repo` scope
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl ShowVersion {
    pub async fn run(self) -> Result<()> {
        let token = handle_token(self.token.as_deref()).await?;
        let github = GitHub::new(&token)?;

        // Get a list of all versions for the given package
        let mut versions = github.get_versions(&self.package_identifier).await?;

        // Get the manifests for the latest or specified version
        let manifests = github
            .get_manifests(
                &self.package_identifier,
                &self
                    .package_version
                    .unwrap_or_else(|| versions.pop_last().unwrap_or_else(|| unreachable!())),
            )
            .await?;

        let all = matches!(
            (
                self.installer_manifest,
                self.default_locale_manifest,
                self.locale_manifests,
                self.version_manifest
            ),
            (false, false, false, false)
        );

        let mut contents = Vec::new();
        if all || self.installer_manifest {
            contents.push(serde_yaml::to_string(&manifests.installer)?);
        }
        if all || self.default_locale_manifest {
            contents.push(serde_yaml::to_string(&manifests.default_locale)?);
        }
        if all || self.locale_manifests {
            contents.extend(
                manifests
                    .locales
                    .into_iter()
                    .flat_map(|locale_manifest| serde_yaml::to_string(&locale_manifest)),
            );
        }
        if all || self.version_manifest {
            contents.push(serde_yaml::to_string(&manifests.version)?);
        }

        print_changes(contents.iter().map(String::as_str));

        Ok(())
    }
}
