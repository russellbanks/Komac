use std::{
    collections::BTreeSet,
    num::{NonZeroU32, NonZeroUsize},
    path::PathBuf,
};

use anstream::println;
use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::eyre::{Error, Result, bail};
use futures_util::TryFutureExt;
use indicatif::ProgressBar;
use owo_colors::OwoColorize;
use secrecy::SecretString;
use strsim::levenshtein;
use tokio::try_join;
use winget_types::{
    InstallerManifest, PackageIdentifier, PackageVersion,
    installer::{Installer, InstallerType, NestedInstallerFiles},
    url::{DecodedUrl, ReleaseNotesUrl},
};

use crate::{
    commands::utils::{SPINNER_TICK_RATE, SubmitOption},
    download::Downloader,
    github::{
        GITHUB_HOST, GitHubError, WINGET_PKGS_FULL_NAME,
        client::{GitHub, GitHubValues},
        utils::{PackagePath, pull_request::Change},
    },
    manifests::{Url, print_changes},
    token::TokenManager,
    traits::path::NormalizePath,
};

/// Add a version to a pre-existing package
#[expect(clippy::struct_excessive_bools)]
#[derive(Parser)]
pub struct UpdateVersion {
    /// The package's unique identifier
    #[arg(value_name = "PACKAGE_IDENTIFIER")]
    identifier: PackageIdentifier,

    /// The package's version
    #[arg(short = 'v', long = "version")]
    version: PackageVersion,

    /// The list of package installers
    #[arg(short, long, num_args = 1.., required = true, value_hint = clap::ValueHint::Url)]
    urls: Vec<Url>,

    /// Number of installers to download at the same time
    #[arg(long, default_value_t = NonZeroUsize::new(num_cpus::get()).unwrap())]
    concurrent_downloads: NonZeroUsize,

    /// List of issues that updating this package would resolve
    #[arg(long)]
    resolves: Vec<NonZeroU32>,

    /// Automatically submit a pull request
    #[arg(short, long)]
    submit: bool,

    /// URL to package's release notes
    #[arg(long, value_hint = clap::ValueHint::Url)]
    release_notes_url: Option<ReleaseNotesUrl>,

    /// Name of external tool that invoked Komac
    #[arg(long, env = "KOMAC_CREATED_WITH")]
    created_with: Option<String>,

    /// URL to external tool that invoked Komac
    #[arg(long, env = "KOMAC_CREATED_WITH_URL", value_hint = clap::ValueHint::Url)]
    created_with_url: Option<DecodedUrl>,

    /// Directory to output the manifests to
    #[arg(short, long, env = "OUTPUT_DIRECTORY", value_hint = clap::ValueHint::DirPath)]
    output: Option<PathBuf>,

    /// Open pull request link automatically
    #[arg(long, env = "OPEN_PR")]
    open_pr: bool,

    /// Run without submitting
    #[arg(long, env = "DRY_RUN")]
    dry_run: bool,

    /// Package version to replace
    #[arg(short, long, num_args = 0..=1, default_missing_value = "latest")]
    replace: Option<PackageVersion>,

    /// Skip checking for existing pull requests
    #[arg(long, env)]
    skip_pr_check: bool,

    /// GitHub personal access token with the `public_repo` scope
    #[arg(short, long, env = "GITHUB_TOKEN", hide_env_values = true)]
    token: Option<SecretString>,
}

impl UpdateVersion {
    pub async fn run(mut self) -> Result<()> {
        let token_manager = TokenManager::handle(self.token.take()).await?;
        let github = GitHub::new(&token_manager)?;

        let mut package = github
            .get_versioned_package(&self.identifier, &self.version)
            .await?;

        println!(
            "Latest version of {}: {}",
            self.identifier,
            package.latest_version()
        );

        if !self.skip_pr_check && !self.dry_run && !package.prompt_existing_pr()? {
            return Ok(());
        }

        let replace_version = self
            .resolve_replace_version(package.versions(), package.latest_version())?
            .cloned();

        let downloader = Downloader::new_with_concurrent(self.concurrent_downloads)?;
        let (mut github_values, mut files) = try_join!(
            self.fetch_github_values(&github).map_err(Error::new),
            downloader.download(self.urls.iter().cloned()),
        )?;

        let manifests = package.manifests_mut().unwrap();

        let download_results = files.analyze().await?;

        let installers = download_results
            .into_values()
            .flat_map(|analyzer| {
                let possible_installer_files = analyzer
                    .zip
                    .as_ref()
                    .map(|zip| zip.possible_installer_files.clone())
                    .unwrap_or_default();
                let mut installers = analyzer.into_installers();
                for installer in &mut installers {
                    carry_over_nested_installer(
                        installer,
                        &manifests.installer,
                        &possible_installer_files,
                    );
                }
                installers
            })
            .collect();

        manifests.installer.package_version = self.version.clone();
        manifests.installer.installers = installers;

        manifests
            .installer
            .installers
            .iter_mut()
            .flat_map(|installer| &mut installer.apps_and_features_entries)
            .for_each(|entry| entry.deduplicate(&manifests.default_locale));

        manifests.installer.optimize();

        manifests.update(
            &self.version,
            &mut github_values,
            self.release_notes_url.as_ref(),
        );

        let mut changes = manifests.create(
            &self.identifier,
            &self.version,
            self.created_with.as_deref(),
        );

        if self.dry_run {
            print_changes(changes.iter().map(Change::manifest));
            return Ok(());
        }

        let submit_option =
            SubmitOption::prompt(&mut changes, &self.identifier, &self.version, self.submit)?;

        let package_path = PackagePath::new(&self.identifier, Some(&self.version), None);
        if let Some(output) = self
            .output
            .as_ref()
            .map(|out| out.join(package_path.as_str()))
        {
            changes.write_to(output.as_path()).await?;
            println!(
                "{} written all manifest files to {}",
                "Successfully".green(),
                output.display()
            );
        }

        if submit_option.is_exit() {
            return Ok(());
        }

        // Create an indeterminate progress bar to show as a pull request is being created
        let pr_progress = ProgressBar::new_spinner().with_message(format!(
            "Creating a pull request for {} {}",
            self.identifier, self.version
        ));
        pr_progress.enable_steady_tick(SPINNER_TICK_RATE);

        let pull_request = github
            .add_version()
            .identifier(&self.identifier)
            .version(&self.version)
            .versions(package.versions())
            .changes(changes)
            .maybe_replace_version(replace_version.as_ref())
            .issue_resolves(&self.resolves)
            .maybe_created_with(self.created_with.as_deref())
            .maybe_created_with_url(self.created_with_url.as_ref())
            .send()
            .await?;

        pr_progress.finish_and_clear();

        pull_request.print_success();

        if self.open_pr {
            open::that(pull_request.url().as_str())?;
        }

        Ok(())
    }

    fn resolve_replace_version<'a>(
        &'a self,
        versions: &'a BTreeSet<PackageVersion>,
        latest_version: &'a PackageVersion,
    ) -> Result<Option<&'a PackageVersion>> {
        let replace_version = self
            .replace
            .as_ref()
            .map(|version| {
                if version.is_latest() {
                    latest_version
                } else {
                    version
                }
            })
            .filter(|&version| version.as_str() != self.version.as_str());

        if let Some(version) = replace_version
            && !versions.contains(version)
            && let Some(closest) = version.closest(versions)
        {
            bail!(
                "Replacement version {version} does not exist in {WINGET_PKGS_FULL_NAME}. The closest version is {closest}"
            );
        }

        Ok(replace_version)
    }

    async fn fetch_github_values(
        &self,
        github: &GitHub,
    ) -> Result<Option<GitHubValues>, GitHubError> {
        if let Some(url) = self
            .urls
            .iter()
            .find(|url| url.host_str() == Some(GITHUB_HOST))
        {
            github
                .get_all_values_from_url(url.clone().into_inner())
                .await
                .transpose()
        } else {
            Ok(None)
        }
    }
}

/// Fills in the nested installer of a zip installer that couldn't be detected, such as when the zip
/// contains more than one executable, from the previous installer of the same architecture.
fn carry_over_nested_installer(
    installer: &mut Installer,
    previous_manifest: &InstallerManifest,
    possible_installer_files: &[Utf8PathBuf],
) {
    if installer.r#type != Some(InstallerType::Zip) || !installer.nested_installer_files.is_empty()
    {
        return;
    }

    let Some(previous_installer) = previous_manifest.installers.iter().find(|previous| {
        previous.architecture == installer.architecture
            && previous.r#type.or(previous_manifest.r#type) == Some(InstallerType::Zip)
    }) else {
        return;
    };

    let previous_nested_files = if previous_installer.nested_installer_files.is_empty() {
        &previous_manifest.nested_installer_files
    } else {
        &previous_installer.nested_installer_files
    };

    installer.nested_installer_files = previous_nested_files
        .iter()
        .map(|nested_file| fix_relative_path(nested_file, possible_installer_files))
        .collect();

    if installer.nested_installer_type.is_none() {
        installer.nested_installer_type = previous_installer
            .nested_installer_type
            .or(previous_manifest.nested_installer_type);
    }
}

/// Points a nested file at the closest matching file in the new zip, as paths often contain the
/// version.
fn fix_relative_path(
    nested_file: &NestedInstallerFiles,
    possible_installer_files: &[Utf8PathBuf],
) -> NestedInstallerFiles {
    let relative_file_path = nested_file.relative_file_path.normalize();
    if possible_installer_files.contains(&relative_file_path) {
        return nested_file.clone();
    }

    possible_installer_files
        .iter()
        .min_by_key(|file_path| levenshtein(file_path.as_str(), relative_file_path.as_str()))
        .map_or_else(
            || nested_file.clone(),
            |file_path| NestedInstallerFiles {
                relative_file_path: file_path.clone(),
                ..nested_file.clone()
            },
        )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use camino::Utf8PathBuf;
    use winget_types::{
        InstallerManifest,
        installer::{Architecture, Installer, InstallerType, NestedInstallerFiles},
    };

    use super::{carry_over_nested_installer, fix_relative_path};

    fn nested_files(paths: &[&str]) -> BTreeSet<NestedInstallerFiles> {
        paths
            .iter()
            .map(|path| NestedInstallerFiles {
                relative_file_path: Utf8PathBuf::from(*path),
                portable_command_alias: None,
            })
            .collect()
    }

    fn zip_installer(architecture: Architecture) -> Installer {
        Installer {
            architecture,
            r#type: Some(InstallerType::Zip),
            ..Installer::default()
        }
    }

    fn previous_manifest() -> InstallerManifest {
        InstallerManifest {
            r#type: Some(InstallerType::Zip),
            nested_installer_type: InstallerType::Portable.try_into().ok(),
            installers: vec![
                Installer {
                    nested_installer_files: nested_files(&["app-x64/app.exe", "app-x64/cli.exe"]),
                    ..zip_installer(Architecture::X64)
                },
                Installer {
                    nested_installer_files: nested_files(&[
                        "app-arm64/app.exe",
                        "app-arm64/cli.exe",
                    ]),
                    ..zip_installer(Architecture::Arm64)
                },
            ],
            ..InstallerManifest::default()
        }
    }

    #[test]
    fn carries_over_nested_files_of_same_architecture() {
        let mut installer = zip_installer(Architecture::Arm64);
        let possible_files = [
            Utf8PathBuf::from("app-arm64/app.exe"),
            Utf8PathBuf::from("app-arm64/cli.exe"),
        ];

        let previous = previous_manifest();

        carry_over_nested_installer(&mut installer, &previous, &possible_files);

        assert_eq!(
            installer.nested_installer_files,
            nested_files(&["app-arm64/app.exe", "app-arm64/cli.exe"])
        );
        assert!(previous.nested_installer_type.is_some());
        assert_eq!(
            installer.nested_installer_type,
            previous.nested_installer_type
        );
    }

    #[test]
    fn carries_over_root_nested_files() {
        let previous = InstallerManifest {
            nested_installer_files: nested_files(&["app.exe", "cli.exe"]),
            installers: vec![zip_installer(Architecture::X64)],
            ..previous_manifest()
        };
        let mut installer = zip_installer(Architecture::X64);

        carry_over_nested_installer(
            &mut installer,
            &previous,
            &[Utf8PathBuf::from("app.exe"), Utf8PathBuf::from("cli.exe")],
        );

        assert_eq!(
            installer.nested_installer_files,
            nested_files(&["app.exe", "cli.exe"])
        );
    }

    #[test]
    fn keeps_detected_nested_files() {
        let mut installer = Installer {
            nested_installer_files: nested_files(&["detected.exe"]),
            ..zip_installer(Architecture::X64)
        };

        carry_over_nested_installer(&mut installer, &previous_manifest(), &[]);

        assert_eq!(
            installer.nested_installer_files,
            nested_files(&["detected.exe"])
        );
    }

    #[test]
    fn ignores_non_zip_installers() {
        let mut installer = Installer {
            architecture: Architecture::X64,
            r#type: Some(InstallerType::Wix),
            ..Installer::default()
        };

        carry_over_nested_installer(&mut installer, &previous_manifest(), &[]);

        assert!(installer.nested_installer_files.is_empty());
        assert_eq!(installer.nested_installer_type, None);
    }

    #[test]
    fn fixes_versioned_relative_path() {
        let possible_files = [
            Utf8PathBuf::from("app_1.1.0/app.exe"),
            Utf8PathBuf::from("app_1.1.0/cli.exe"),
        ];
        let nested_file = NestedInstallerFiles {
            relative_file_path: Utf8PathBuf::from("app_1.0.0/cli.exe"),
            portable_command_alias: None,
        };

        assert_eq!(
            fix_relative_path(&nested_file, &possible_files).relative_file_path,
            "app_1.1.0/cli.exe"
        );
    }
}
