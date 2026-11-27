use clap::Parser;
use color_eyre::Result;
use secrecy::SecretString;
use winget_types::{PackageIdentifier, PackageVersion};

use crate::{github::client::GitHub, i18n::fl, manifests::print_changes, token::TokenManager};

#[expect(clippy::struct_excessive_bools)]
#[derive(Parser)]
#[command(about = fl!("command-show"))]
pub struct ShowVersion {
    #[arg(value_name = "PACKAGE_IDENTIFIER", help = fl!("arg-package-identifier"))]
    package_identifier: PackageIdentifier,

    #[arg(short = 'v', long = "version", help = fl!("arg-package-version"))]
    package_version: Option<PackageVersion>,

    #[arg(short, long, help = fl!("arg-show-installer-manifest"))]
    installer_manifest: bool,

    #[arg(short, long = "defaultlocale-manifest", help = fl!("arg-show-default-locale-manifest"))]
    default_locale_manifest: bool,

    #[arg(short, long, help = fl!("arg-show-locale-manifests"))]
    locale_manifests: bool,

    #[arg(long, help = fl!("arg-show-version-manifest"))]
    version_manifest: bool,

    #[arg(short, long, env = "GITHUB_TOKEN", hide_env_values = true, help = fl!("arg-token"))]
    token: Option<SecretString>,
}

impl ShowVersion {
    pub async fn run(self) -> Result<()> {
        let token_manager = TokenManager::handle(self.token).await?;
        let github = GitHub::new(&token_manager)?;

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

        print_changes(contents);

        Ok(())
    }
}
