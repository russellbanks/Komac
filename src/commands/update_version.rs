use std::{
    collections::BTreeSet,
    io::{IsTerminal, stdin},
    num::{NonZeroU32, NonZeroUsize},
    path::PathBuf,
};

use anstream::println;
use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::eyre::{Error, Result, bail};
use futures_util::TryFutureExt;
use indicatif::ProgressBar;
use secrecy::SecretString;
use strsim::levenshtein;
use tokio::try_join;
use tracing::warn;
use winget_types::{
    InstallerManifest, PackageIdentifier, PackageVersion,
    installer::{Installer, InstallerType, NestedInstallerFiles},
    url::{DecodedUrl, ReleaseNotesUrl},
};

use crate::{
    commands::utils::{SPINNER_TICK_RATE, SubmitOption, environment::CI},
    download::Downloader,
    github::{
        GITHUB_HOST, GitHubError,
        client::{GitHub, GitHubValues},
        upstream,
        utils::{PackagePath, pull_request::Change},
    },
    i18n::fl,
    manifests::{Url, print_changes},
    prompts::text::optional_prompt,
    terminal::success,
    token::TokenManager,
    traits::path::NormalizePath,
};

#[expect(clippy::struct_excessive_bools)]
#[derive(Parser)]
#[command(about = fl!("command-update"))]
pub struct UpdateVersion {
    #[arg(value_name = "PACKAGE_IDENTIFIER", help = fl!("arg-identifier"))]
    identifier: PackageIdentifier,

    #[arg(short = 'v', long = "version", help = fl!("arg-version"))]
    version: PackageVersion,

    #[arg(
        short,
        long,
        num_args = 1..,
        required = true,
        value_hint = clap::ValueHint::Url,
        help = fl!("arg-urls"),
    )]
    urls: Vec<Url>,

    #[arg(
        long,
        default_value_t = std::thread::available_parallelism().unwrap_or(NonZeroUsize::MIN),
        help = fl!("arg-concurrent-downloads"),
    )]
    concurrent_downloads: NonZeroUsize,

    #[arg(long, help = fl!("arg-update-resolves"))]
    resolves: Vec<NonZeroU32>,

    #[arg(short, long, help = fl!("arg-submit"))]
    submit: bool,

    #[arg(long, conflicts_with_all = ["submit", "dry_run"], help = fl!("arg-exit"))]
    exit: bool,

    #[arg(long, value_hint = clap::ValueHint::Url, help = fl!("arg-update-release-notes-url"))]
    release_notes_url: Option<ReleaseNotesUrl>,

    #[arg(long, env = "KOMAC_CREATED_WITH", help = fl!("arg-created-with"))]
    created_with: Option<String>,

    #[arg(
        long,
        env = "KOMAC_CREATED_WITH_URL",
        value_hint = clap::ValueHint::Url,
        help = fl!("arg-created-with-url"),
    )]
    created_with_url: Option<DecodedUrl>,

    #[arg(
        short,
        long,
        env = "OUTPUT_DIRECTORY",
        value_hint = clap::ValueHint::DirPath,
        help = fl!("arg-output"),
    )]
    output: Option<PathBuf>,

    #[arg(long, env = "OPEN_PR", help = fl!("arg-open-pr"))]
    open_pr: bool,

    #[arg(long, env = "DRY_RUN", help = fl!("arg-dry-run"))]
    dry_run: bool,

    #[arg(
        short,
        long,
        num_args = 0..=1,
        default_missing_value = "latest",
        help = fl!("arg-update-replace"),
    )]
    replace: Option<PackageVersion>,

    #[arg(long, env, help = fl!("arg-skip-pr-check"))]
    skip_pr_check: bool,

    #[arg(short, long, env = "GITHUB_TOKEN", hide_env_values = true, help = fl!("arg-token"))]
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
            "{}",
            fl!(
                "latest-version",
                identifier = self.identifier.as_str(),
                version = package.latest_version().to_string()
            )
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

        // The previous version had a release notes URL that would otherwise be dropped
        if self.release_notes_url.is_none()
            && github_values
                .as_ref()
                .and_then(GitHubValues::release_notes_url)
                .is_none()
            && let Some(previous_url) = &manifests.default_locale.release_notes_url
        {
            if self.is_interactive() {
                self.release_notes_url = optional_prompt(
                    None,
                    Some(suggest_release_notes_url(
                        previous_url.as_str(),
                        manifests.default_locale.package_version.as_str(),
                        self.version.as_str(),
                    )),
                )?;
            } else {
                warn!(
                    "The previous version had a release notes URL ({previous_url}). Use --release-notes-url to set one for {}",
                    self.version
                );
            }
        }

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

        let submit_option = SubmitOption::prompt(
            &mut changes,
            &self.identifier,
            &self.version,
            self.submit,
            self.exit,
        )?;

        let package_path = PackagePath::new(&self.identifier, Some(&self.version), None);
        if let Some(output) = self
            .output
            .as_ref()
            .map(|out| out.join(package_path.as_str()))
        {
            changes.write_to(output.as_path()).await?;
            println!(
                "{}",
                fl!(
                    "manifests-written",
                    success = success(),
                    path = output.display().to_string()
                )
            );
        }

        if submit_option.is_exit() {
            return Ok(());
        }

        // Create an indeterminate progress bar to show as a pull request is being created
        let pr_progress = ProgressBar::new_spinner().with_message(fl!(
            "pull-request-creating",
            identifier = self.identifier.as_str(),
            version = self.version.to_string()
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

    fn is_interactive(&self) -> bool {
        !self.submit && !self.exit && !self.dry_run && !*CI && stdin().is_terminal()
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
                "Replacement version {version} does not exist in {}. The closest version is {closest}",
                upstream()
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

/// Suggests a release notes URL for the new version by replacing occurrences of the previous
/// version in the previous URL. Occurrences that are part of a longer version, such as `1.2` in
/// `1.2.3`, are left untouched. A URL without the previous version is returned as is, as it's
/// likely to be shared between versions.
fn suggest_release_notes_url(
    previous_url: &str,
    previous_version: &str,
    new_version: &str,
) -> String {
    let is_digit = |char: char| char.is_ascii_digit();
    let mut suggested = String::with_capacity(previous_url.len());
    let mut last_end = 0;

    for (start, _) in previous_url.match_indices(previous_version) {
        let end = start + previous_version.len();
        let before = &previous_url[..start];
        let after = &previous_url[end..];
        let continues_before = before
            .strip_suffix('.')
            .unwrap_or(before)
            .ends_with(is_digit);
        let continues_after = after
            .strip_prefix('.')
            .unwrap_or(after)
            .starts_with(is_digit);

        if !continues_before && !continues_after {
            suggested.push_str(&previous_url[last_end..start]);
            suggested.push_str(new_version);
            last_end = end;
        }
    }

    suggested.push_str(&previous_url[last_end..]);
    suggested
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
    use rstest::rstest;
    use winget_types::{
        InstallerManifest,
        installer::{Architecture, Installer, InstallerType, NestedInstallerFiles},
    };

    use super::{carry_over_nested_installer, fix_relative_path, suggest_release_notes_url};

    #[rstest]
    #[case(
        "https://github.com/owner/repo/releases/tag/v1.2.3",
        "1.2.3",
        "1.3.0",
        "https://github.com/owner/repo/releases/tag/v1.3.0"
    )]
    #[case(
        "https://example.com/app_1.2.3/notes-1.2.3.html",
        "1.2.3",
        "2.0.0",
        "https://example.com/app_2.0.0/notes-2.0.0.html"
    )]
    #[case(
        "https://docs.nvidia.com/cuda/cuda-toolkit-release-notes/index.html",
        "13.0.2",
        "13.1.0",
        "https://docs.nvidia.com/cuda/cuda-toolkit-release-notes/index.html"
    )]
    #[case(
        "https://example.com/releases/1.2.30",
        "1.2.3",
        "1.2.4",
        "https://example.com/releases/1.2.30"
    )]
    #[case(
        "https://example.com/v5/releases/15",
        "5",
        "6",
        "https://example.com/v6/releases/15"
    )]
    fn suggests_release_notes_url(
        #[case] previous_url: &str,
        #[case] previous_version: &str,
        #[case] new_version: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(
            suggest_release_notes_url(previous_url, previous_version, new_version),
            expected
        );
    }

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
