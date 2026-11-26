use std::{fs::File, io, num::NonZeroU32};

use anstream::println;
use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::Result;
use indicatif::ProgressBar;
use inquire::MultiSelect;
use itertools::Itertools;
use owo_colors::OwoColorize;
use secrecy::SecretString;
use tracing::warn;
use walkdir::WalkDir;
use winget_types::{Manifest as WingetManifest, ManifestType, utils::GenericManifest};

use crate::{
    commands::utils::{RateLimit, SPINNER_TICK_RATE, SubmitOption},
    github::{client::GitHub, utils::pull_request::Change},
    i18n::fl,
    manifests::{Manifests, manifest::Manifest, print_changes},
    prompts::handle_inquire_error,
    token::TokenManager,
};

#[expect(clippy::struct_excessive_bools, reason = "CLI flags")]
#[derive(Parser)]
pub struct Submit {
    #[arg(value_hint = clap::ValueHint::DirPath)]
    path: Utf8PathBuf,

    #[arg(
        short = 'y',
        long = "yes",
        visible_alias = "submit",
        help = fl!("arg-submit-skip-prompt"),
    )]
    skip_prompt: bool,

    #[arg(short = 'a', long = "all", help = fl!("arg-submit-submit-all"))]
    submit_all: bool,

    #[arg(long, help = fl!("arg-submit-resolves"))]
    resolves: Vec<NonZeroU32>,

    #[arg(long, env = "OPEN_PR", help = fl!("arg-open-pr"))]
    open_pr: bool,

    #[arg(long, hide = true, help = fl!("arg-fast"))]
    fast: bool,

    #[arg(long, env = "DRY_RUN", help = fl!("arg-dry-run"))]
    dry_run: bool,

    #[arg(short, long, env = "GITHUB_TOKEN", hide_env_values = true, help = fl!("arg-token"))]
    token: Option<SecretString>,
}

impl Submit {
    pub async fn run(mut self) -> Result<()> {
        let token_manager = TokenManager::handle(self.token.take()).await?;

        let yaml_entries = self.get_yaml_file_paths()?;

        let packages = yaml_entries
            .iter()
            .filter_map(|path| {
                Self::read_manifest(path)
                    .inspect_err(|error| warn!("Skipping {path}: {error}"))
                    .ok()
            })
            .chunk_by(|manifest| {
                // Group manifests by both the package identifier and the package version
                (
                    manifest.package_identifier().clone(),
                    manifest.package_version().clone(),
                )
            })
            .into_iter()
            .filter_map(|(_, manifests)| {
                // Now that we solely have manifests related to a package and its version, we can
                // rebuild it into a Manifests struct
                let mut installer = None;
                let mut default_locale = None;
                let mut locales = Vec::new();
                let mut version = None;
                for manifest in manifests {
                    match manifest {
                        Manifest::Installer(installer_manifest) => {
                            installer = Some(installer_manifest);
                        }
                        Manifest::DefaultLocale(default_locale_manifest) => {
                            default_locale = Some(default_locale_manifest);
                        }
                        Manifest::Locale(locale) => locales.push(locale),
                        Manifest::Version(version_manifest) => version = Some(version_manifest),
                    }
                }
                Some(Manifests {
                    installer: installer?,
                    default_locale: default_locale?,
                    locales,
                    version: version?,
                })
            })
            .collect::<Vec<_>>();

        // If there's only one package, use that. Otherwise, prompt for which packages to submit
        let manifests = match packages.len() {
            0 => {
                println!(
                    "{}",
                    fl!("no-packages-to-submit", path = self.path.blue().to_string())
                );
                return Ok(());
            }
            1 => packages,
            _ if self.submit_all => packages,
            _ => MultiSelect::new(&fl!("prompt-select-packages"), packages)
                .with_page_size(10)
                .prompt()
                .map_err(handle_inquire_error)?,
        };

        let rate_limit = RateLimit::new(self.fast);

        let github = GitHub::new(token_manager)?;

        for mut package_manifests in manifests {
            // Reorder the keys in case the manifests weren't created by komac
            package_manifests.installer.optimize();

            package_manifests.default_locale.update_manifest_version();
            package_manifests.version.update_manifest_version();
            for locale_manifest in &mut package_manifests.locales {
                locale_manifest.update_manifest_version();
            }

            let identifier = package_manifests.package_identifier();
            let version = package_manifests.package_version();

            let mut changes = package_manifests.create(identifier, version, None);

            if self.dry_run {
                print_changes(changes.iter().map(Change::manifest));
                continue;
            }

            let submit_option =
                SubmitOption::prompt(&mut changes, identifier, version, self.skip_prompt, false)?;

            if submit_option.is_exit() {
                continue;
            }

            let versions = github.get_versions(identifier).await.ok();

            rate_limit.wait().await;

            // Create an indeterminate progress bar to show as a pull request is being created
            let pr_progress = ProgressBar::new_spinner().with_message(fl!(
                "pull-request-creating",
                identifier = identifier.as_str(),
                version = version.to_string()
            ));
            pr_progress.enable_steady_tick(SPINNER_TICK_RATE);

            let pull_request = github
                .add_version()
                .identifier(identifier)
                .version(version)
                .maybe_versions(versions.as_ref())
                .changes(changes)
                .issue_resolves(&self.resolves)
                .send()
                .await?;

            rate_limit.record().await;

            pr_progress.finish_and_clear();

            pull_request.print_success();

            if self.open_pr {
                open::that(pull_request.url().as_str())?;
            }
        }

        Ok(())
    }

    /// Reads a YAML file and deserializes it into the manifest type it declares.
    fn read_manifest(path: &Utf8PathBuf) -> Result<Manifest> {
        // Read file to string so we can read it twice - once for the manifest type and
        // second for the full manifest
        let manifest = io::read_to_string(File::open(path)?)?;

        // Deserialize the manifest into just the manifest type so that it can be determined
        // which manifest to properly deserialize into
        Ok(
            match serde_yaml::from_str::<GenericManifest>(&manifest)?.r#type {
                ManifestType::Installer => Manifest::Installer(serde_yaml::from_str(&manifest)?),
                ManifestType::DefaultLocale => {
                    Manifest::DefaultLocale(serde_yaml::from_str(&manifest)?)
                }
                ManifestType::Locale => Manifest::Locale(serde_yaml::from_str(&manifest)?),
                ManifestType::Version => Manifest::Version(serde_yaml::from_str(&manifest)?),
            },
        )
    }

    fn get_yaml_file_paths(&self) -> walkdir::Result<Vec<Utf8PathBuf>> {
        WalkDir::new(&self.path)
            .into_iter()
            .filter_map_ok(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("yaml"))
                    .then(|| Utf8PathBuf::from_path_buf(entry.into_path()).ok())?
            })
            .collect::<walkdir::Result<Vec<_>>>()
    }
}
