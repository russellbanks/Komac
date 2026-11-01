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
    installer::{Architecture, Installer, NestedInstallerFiles, Switches},
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

        let previous_installers = PreviousInstallers::new(&manifests.installer);
        let installers = download_results
            .into_values()
            .flat_map(|analyzer| {
                let possible_installer_files = analyzer
                    .zip
                    .as_ref()
                    .map(|zip| zip.possible_installer_files.clone())
                    .unwrap_or_default();
                analyzer
                    .into_installers()
                    .into_iter()
                    .flat_map(|installer| {
                        merge_with_previous(
                            installer,
                            &previous_installers,
                            &possible_installer_files,
                        )
                    })
                    .collect::<Vec<_>>()
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

/// The previous version's installers, used to fill in what analysis can't detect.
struct PreviousInstallers {
    /// Each installer with the fields it inherits from the root of the manifest applied.
    installers: Vec<Installer>,

    /// The fields from the root of the manifest that apply to the whole package rather than to a
    /// type of installer, for installers that don't match a previous one.
    package_wide: Installer,
}

impl PreviousInstallers {
    fn new(manifest: &InstallerManifest) -> Self {
        macro_rules! root_installer {
            ($($field:ident),* $(,)?) => {
                Installer {
                    $($field: manifest.$field.clone(),)*
                    ..Installer::default()
                }
            };
        }

        let root = root_installer!(
            locale,
            platform,
            minimum_os_version,
            r#type,
            nested_installer_type,
            nested_installer_files,
            scope,
            install_modes,
            switches,
            success_codes,
            expected_return_codes,
            upgrade_behavior,
            commands,
            protocols,
            file_extensions,
            dependencies,
            package_family_name,
            product_code,
            capabilities,
            restricted_capabilities,
            markets,
            aborts_terminal,
            release_date,
            install_location_required,
            require_explicit_upgrade,
            display_install_warnings,
            unsupported_os_architectures,
            unsupported_arguments,
            apps_and_features_entries,
            elevation_requirement,
            installation_metadata,
            download_command_prohibited,
            repair_behavior,
            archive_binaries_depend_on_path,
        );

        Self {
            installers: manifest
                .installers
                .iter()
                .map(|installer| merge_installer(installer.clone(), root.clone()))
                .collect(),
            package_wide: Installer {
                platform: root.platform,
                minimum_os_version: root.minimum_os_version,
                commands: root.commands.clone(),
                protocols: root.protocols.clone(),
                file_extensions: root.file_extensions.clone(),
                dependencies: root.dependencies,
                ..Installer::default()
            },
        }
    }
}

/// Fills in what analysis couldn't detect for an installer from the previous installer it matches.
///
/// An installer matches a previous installer of the same type, scope, and architecture. If its
/// architecture couldn't be detected and the previous version used the same file for several
/// architectures, it's split back into one installer per architecture. Installers without a match,
/// such as a newly added installer type, only get the package-wide fields.
fn merge_with_previous(
    installer: Installer,
    previous: &PreviousInstallers,
    possible_installer_files: &[Utf8PathBuf],
) -> Vec<Installer> {
    let candidates = previous
        .installers
        .iter()
        .filter(|previous| {
            previous.r#type == installer.r#type
                && (installer.scope.is_none()
                    || previous.scope.is_none()
                    || previous.scope == installer.scope)
        })
        .collect::<Vec<_>>();

    let same_architecture = candidates
        .iter()
        .filter(|previous| previous.architecture == installer.architecture)
        .min_by_key(|previous| {
            levenshtein(
                file_name(previous.url.as_str()),
                file_name(installer.url.as_str()),
            )
        });

    let matches = if let Some(previous) = same_architecture {
        vec![*previous]
    } else if installer.architecture == Architecture::Neutral
        && candidates
            .iter()
            .all(|previous| previous.url == candidates[0].url)
    {
        candidates
    } else {
        Vec::new()
    };

    if matches.is_empty() {
        return vec![merge_installer(installer, previous.package_wide.clone())];
    }

    matches
        .into_iter()
        .map(|previous| {
            let detected_nested_files = !installer.nested_installer_files.is_empty();
            let mut merged = merge_installer(installer.clone(), without_version_specific(previous));
            merged.architecture = previous.architecture;
            if !detected_nested_files {
                merged.nested_installer_files = merged
                    .nested_installer_files
                    .iter()
                    .map(|nested_file| fix_relative_path(nested_file, possible_installer_files))
                    .collect();
            }
            merged
        })
        .collect()
}

/// Fills the fields of `installer` that are unset from `other`, keeping the switches of `installer`
/// if it has any.
fn merge_installer(installer: Installer, mut other: Installer) -> Installer {
    if !installer.switches.is_empty() {
        other.switches = Switches::default();
    }
    installer.merge_with(other)
}

/// Clears the fields of a previous installer that change with every version.
fn without_version_specific(previous: &Installer) -> Installer {
    Installer {
        product_code: None,
        apps_and_features_entries: Default::default(),
        release_date: None,
        ..previous.clone()
    }
}

fn file_name(url: &str) -> &str {
    url.rsplit('/').next().unwrap_or(url)
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
    use std::{collections::BTreeSet, str::FromStr};

    use camino::Utf8PathBuf;
    use winget_types::{
        InstallerManifest,
        installer::{
            Architecture, Installer, InstallerType, MinimumOSVersion, NestedInstallerFiles, Scope,
            UpgradeBehavior,
        },
        url::DecodedUrl,
    };

    use super::{PreviousInstallers, fix_relative_path, merge_with_previous};

    fn nested_files(paths: &[&str]) -> BTreeSet<NestedInstallerFiles> {
        paths
            .iter()
            .map(|path| NestedInstallerFiles {
                relative_file_path: Utf8PathBuf::from(*path),
                portable_command_alias: None,
            })
            .collect()
    }

    fn installer(r#type: InstallerType, architecture: Architecture, url: &str) -> Installer {
        Installer {
            architecture,
            r#type: Some(r#type),
            url: DecodedUrl::from_str(url).unwrap(),
            ..Installer::default()
        }
    }

    fn files(paths: &[&str]) -> Vec<Utf8PathBuf> {
        paths.iter().map(Utf8PathBuf::from).collect()
    }

    #[test]
    fn carries_over_nested_files_of_same_architecture() {
        let previous = InstallerManifest {
            r#type: Some(InstallerType::Zip),
            nested_installer_type: InstallerType::Portable.try_into().ok(),
            installers: vec![
                Installer {
                    nested_installer_files: nested_files(&["app-x64/app.exe", "app-x64/cli.exe"]),
                    ..installer(
                        InstallerType::Zip,
                        Architecture::X64,
                        "https://e.com/1.0/app-x64.zip",
                    )
                },
                Installer {
                    nested_installer_files: nested_files(&[
                        "app-arm64/app.exe",
                        "app-arm64/cli.exe",
                    ]),
                    ..installer(
                        InstallerType::Zip,
                        Architecture::Arm64,
                        "https://e.com/1.0/app-arm64.zip",
                    )
                },
            ],
            ..InstallerManifest::default()
        };

        let merged = merge_with_previous(
            installer(
                InstallerType::Zip,
                Architecture::Arm64,
                "https://e.com/1.1/app-arm64.zip",
            ),
            &PreviousInstallers::new(&previous),
            &files(&["app-arm64/app.exe", "app-arm64/cli.exe"]),
        );

        assert_eq!(merged.len(), 1);
        assert_eq!(
            merged[0].nested_installer_files,
            nested_files(&["app-arm64/app.exe", "app-arm64/cli.exe"])
        );
        assert_eq!(
            merged[0].nested_installer_type,
            previous.nested_installer_type
        );
    }

    #[test]
    fn carries_over_root_fields() {
        let previous = InstallerManifest {
            upgrade_behavior: Some(UpgradeBehavior::UninstallPrevious),
            installers: vec![installer(
                InstallerType::Exe,
                Architecture::X64,
                "https://e.com/1.0/setup.exe",
            )],
            ..InstallerManifest::default()
        };

        let merged = merge_with_previous(
            installer(
                InstallerType::Exe,
                Architecture::X64,
                "https://e.com/1.1/setup.exe",
            ),
            &PreviousInstallers::new(&previous),
            &[],
        );

        assert_eq!(
            merged[0].upgrade_behavior,
            Some(UpgradeBehavior::UninstallPrevious)
        );
    }

    #[test]
    fn splits_neutral_installer_by_previous_architectures() {
        let url = "https://e.com/app.zip";
        let previous = InstallerManifest {
            installers: vec![
                Installer {
                    nested_installer_files: nested_files(&["app.exe"]),
                    ..installer(InstallerType::Zip, Architecture::X86, url)
                },
                Installer {
                    nested_installer_files: nested_files(&["app64.exe"]),
                    ..installer(InstallerType::Zip, Architecture::X64, url)
                },
            ],
            ..InstallerManifest::default()
        };

        let merged = merge_with_previous(
            installer(InstallerType::Zip, Architecture::Neutral, url),
            &PreviousInstallers::new(&previous),
            &files(&["app.exe", "app64.exe"]),
        );

        assert_eq!(
            merged
                .iter()
                .map(|installer| (
                    installer.architecture,
                    installer.nested_installer_files.clone()
                ))
                .collect::<Vec<_>>(),
            vec![
                (Architecture::X86, nested_files(&["app.exe"])),
                (Architecture::X64, nested_files(&["app64.exe"])),
            ]
        );
    }

    #[test]
    fn keeps_installers_without_a_previous_match() {
        let previous = InstallerManifest {
            installers: vec![Installer {
                nested_installer_files: nested_files(&["app.exe"]),
                ..installer(
                    InstallerType::Zip,
                    Architecture::X64,
                    "https://e.com/1.0/app.zip",
                )
            }],
            ..InstallerManifest::default()
        };
        let msi = installer(
            InstallerType::Wix,
            Architecture::X64,
            "https://e.com/1.1/app.msi",
        );

        let merged = merge_with_previous(msi.clone(), &PreviousInstallers::new(&previous), &[]);

        assert_eq!(merged, vec![msi]);
    }

    #[test]
    fn gives_unmatched_installers_package_wide_fields() {
        let minimum_os_version = MinimumOSVersion::from_str("10.0.17763.0").ok();
        let previous = InstallerManifest {
            minimum_os_version,
            upgrade_behavior: Some(UpgradeBehavior::UninstallPrevious),
            installers: vec![installer(
                InstallerType::Zip,
                Architecture::X64,
                "https://e.com/1.0/app.zip",
            )],
            ..InstallerManifest::default()
        };

        let merged = merge_with_previous(
            installer(
                InstallerType::Wix,
                Architecture::X64,
                "https://e.com/1.1/app.msi",
            ),
            &PreviousInstallers::new(&previous),
            &[],
        );

        assert_eq!(merged[0].minimum_os_version, minimum_os_version);
        assert_eq!(merged[0].upgrade_behavior, None);
    }

    #[test]
    fn keeps_detected_values_and_skips_version_specific_ones() {
        let previous = InstallerManifest {
            installers: vec![Installer {
                scope: Some(Scope::Machine),
                product_code: Some(String::from("App 1.0")),
                ..installer(
                    InstallerType::Nullsoft,
                    Architecture::X64,
                    "https://e.com/1.0/setup.exe",
                )
            }],
            ..InstallerManifest::default()
        };
        let new = Installer {
            scope: Some(Scope::Machine),
            upgrade_behavior: Some(UpgradeBehavior::Install),
            ..installer(
                InstallerType::Nullsoft,
                Architecture::X64,
                "https://e.com/1.1/setup.exe",
            )
        };

        let merged = merge_with_previous(new.clone(), &PreviousInstallers::new(&previous), &[]);

        assert_eq!(merged, vec![new]);
    }

    #[test]
    fn fixes_versioned_relative_path() {
        let nested_file = NestedInstallerFiles {
            relative_file_path: Utf8PathBuf::from("app_1.0.0/cli.exe"),
            portable_command_alias: None,
        };

        assert_eq!(
            fix_relative_path(
                &nested_file,
                &files(&["app_1.1.0/app.exe", "app_1.1.0/cli.exe"])
            )
            .relative_file_path,
            "app_1.1.0/cli.exe"
        );
    }
}
