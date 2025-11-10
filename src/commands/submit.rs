use std::{fs::File, io, num::NonZeroU32};

use anstream::println;
use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::{Result, eyre};
use indicatif::ProgressBar;
use inquire::MultiSelect;
use itertools::Itertools;
use owo_colors::OwoColorize;
use walkdir::WalkDir;
use winget_types::{GenericManifest, ManifestType};

use crate::{
    commands::utils::{RateLimit, SPINNER_TICK_RATE, SubmitOption},
    github::{
        WINGET_PKGS_FULL_NAME,
        client::GitHub,
        utils::{PackagePath, pull_request::pr_changes},
    },
    manifests::{Manifests, manifest::Manifest},
    prompts::handle_inquire_error,
    terminal::Hyperlinkable,
    token::TokenManager,
};
#[derive(Parser)]
pub struct Submit {
    #[arg(value_hint = clap::ValueHint::DirPath)]
    path: Utf8PathBuf,

    /// Skip the confirmation prompt to submit the package
    #[arg(short = 'y', long = "yes", visible_alias = "submit")]
    skip_prompt: bool,

    /// Submit all packages present in the folder
    #[arg(short = 'a', long = "all")]
    submit_all: bool,

    /// List of issues that submitting this version would resolve
    #[arg(long)]
    resolves: Vec<NonZeroU32>,

    /// Open pull request link automatically
    #[arg(long, env = "OPEN_PR")]
    open_pr: bool,

    /// Use the per-minute rate limit, potentially hitting the hourly rate limit in 7.5 minutes
    #[arg(long, hide = true)]
    fast: bool,

    /// Run without submitting
    #[arg(long, env = "DRY_RUN")]
    dry_run: bool,

    /// GitHub personal access token with the `public_repo` scope
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl Submit {
    pub async fn run(self) -> Result<()> {
        let token = TokenManager::handle(self.token.as_deref()).await?;

        let yaml_entries = self.get_yaml_file_paths()?;

        let packages = yaml_entries
            .iter()
            .flat_map(|path| {
                // Read file to string so we can read it twice - once for the manifest type and
                // second for the full manifest
                let manifest = io::read_to_string(File::open(path)?)?;

                // Deserialize the manifest into just the manifest type so that it can be determined
                // which manifest to properly deserialize into
                let manifest = match serde_yaml::from_str::<GenericManifest>(&manifest)?.r#type {
                    ManifestType::Installer => {
                        Manifest::Installer(serde_yaml::from_str(&manifest)?)
                    }
                    ManifestType::DefaultLocale => {
                        Manifest::DefaultLocale(serde_yaml::from_str(&manifest)?)
                    }
                    ManifestType::Locale => Manifest::Locale(serde_yaml::from_str(&manifest)?),
                    ManifestType::Version => Manifest::Version(serde_yaml::from_str(&manifest)?),
                };
                Ok::<Manifest, eyre::Error>(manifest)
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
                    "No valid packages to submit were found in {}",
                    self.path.blue()
                );
                return Ok(());
            }
            1 => packages,
            _ if self.submit_all => packages,
            _ => MultiSelect::new("Please select which packages to submit", packages)
                .with_page_size(10)
                .prompt()
                .map_err(handle_inquire_error)?,
        };

        let rate_limit = RateLimit::new(self.fast);

        for mut manifest in manifests {
            let identifier = &manifest.version.package_identifier;
            let version = &manifest.version.package_version;

            // Reorder the keys in case the manifests weren't created by komac
            manifest.installer.optimize();

            let package_path = PackagePath::new(identifier, Some(version), None);
            let mut changes = pr_changes()
                .package_identifier(identifier)
                .manifests(&manifest)
                .package_path(&package_path)
                .create()?;

            let submit_option = SubmitOption::prompt(
                &mut changes,
                identifier,
                version,
                self.skip_prompt,
                self.dry_run,
            )?;

            if submit_option.is_exit() {
                continue;
            }

            let github = GitHub::new(&token)?;
            let versions = github.get_versions(identifier).await.unwrap_or_default();

            rate_limit.wait().await;

            // Create an indeterminate progress bar to show as a pull request is being created
            let pr_progress = ProgressBar::new_spinner().with_message(format!(
                "Creating a pull request for {identifier} version {version}",
            ));
            pr_progress.enable_steady_tick(SPINNER_TICK_RATE);

            let pull_request_url = github
                .add_version()
                .identifier(identifier)
                .version(version)
                .versions(&versions)
                .changes(changes)
                .issue_resolves(&self.resolves)
                .send()
                .await?;

            rate_limit.record().await;

            pr_progress.finish_and_clear();

            println!(
                "{} created a {} to {WINGET_PKGS_FULL_NAME}",
                "Successfully".green(),
                "pull request".hyperlink(&pull_request_url)
            );

            if self.open_pr {
                open::that(pull_request_url.as_str())?;
            }
        }

        Ok(())
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
