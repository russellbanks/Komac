use crate::commands::utils::{prompt_submit_option, SubmitOption, SPINNER_TICK_RATE};
use crate::credential::handle_token;
use crate::github::github_client::{GitHub, WINGET_PKGS_FULL_NAME};
use crate::github::utils::pull_request::pr_changes;
use crate::github::utils::{get_package_path, is_manifest_file};
use crate::manifests::default_locale_manifest::DefaultLocaleManifest;
use crate::manifests::installer_manifest::InstallerManifest;
use crate::manifests::locale_manifest::LocaleManifest;
use crate::manifests::version_manifest::VersionManifest;
use crate::manifests::Manifests;
use crate::prompts::prompt::handle_inquire_error;
use anstream::println;
use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::eyre::bail;
use color_eyre::Result;
use indicatif::ProgressBar;
use inquire::Select;
use itertools::Itertools;
use owo_colors::OwoColorize;
use std::fs::File;
use walkdir::WalkDir;

#[derive(Parser)]
pub struct Submit {
    #[arg(value_hint = clap::ValueHint::DirPath)]
    path: Utf8PathBuf,

    /// Open pull request link automatically
    #[arg(long, env = "OPEN_PR")]
    open_pr: bool,

    /// Skip the confirmation prompt to submit the package
    #[arg(short = 'y', long = "yes")]
    skip_prompt: bool,

    /// Run without submitting
    #[arg(long, env = "DRY_RUN")]
    dry_run: bool,

    /// GitHub personal access token with the `public_repo` scope
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl Submit {
    pub async fn run(self) -> Result<()> {
        let token = handle_token(self.token.as_deref());

        let yaml_entries = self.get_yaml_file_paths()?;

        // Group package manifests together. This is so that if there are multiple packages in the
        // same directory, we can prompt the user for which one to submit.
        let mut packages = yaml_entries
            .iter()
            .filter(|entry| {
                // Get all installer manifest files
                entry
                    .file_name()
                    .is_some_and(|name| name.ends_with(".installer.yaml"))
            })
            .flat_map(File::open)
            .flat_map(serde_yaml::from_reader)
            .filter_map(|installer_manifest: InstallerManifest| {
                // For each installer manifest, get all manifest files for that package
                let package_manifests = yaml_entries
                    .iter()
                    .filter_map(|path| Option::from(path).zip(path.file_name()))
                    .filter(|(_, file_name)| {
                        file_name.starts_with(&*installer_manifest.package_identifier)
                    })
                    .collect::<Vec<_>>();

                // Find and parse the version manifest
                let version_manifest: VersionManifest = package_manifests
                    .iter()
                    .find(|(_, file_name)| {
                        is_manifest_file::<VersionManifest>(
                            file_name,
                            &installer_manifest.package_identifier,
                            None,
                        )
                    })
                    .and_then(|(path, _)| {
                        File::open(path)
                            .ok()
                            .and_then(|file| serde_yaml::from_reader(file).ok())
                    })?;

                // Find and parse the default locale manifest
                let default_locale_manifest = package_manifests
                    .iter()
                    .find(|(_, file_name)| {
                        is_manifest_file::<DefaultLocaleManifest>(
                            file_name,
                            &installer_manifest.package_identifier,
                            Some(&version_manifest.default_locale),
                        )
                    })
                    .and_then(|(path, _)| {
                        File::open(path)
                            .ok()
                            .and_then(|file| serde_yaml::from_reader(file).ok())
                    })?;

                // Find and parse any locale manifests
                let locale_manifests = package_manifests
                    .iter()
                    .filter(|(_, file_name)| {
                        is_manifest_file::<LocaleManifest>(
                            file_name,
                            &installer_manifest.package_identifier,
                            Some(&version_manifest.default_locale),
                        )
                    })
                    .flat_map(|(path, _)| File::open(path))
                    .flat_map(serde_yaml::from_reader)
                    .collect::<Vec<_>>();

                Some(Manifests {
                    installer: installer_manifest,
                    default_locale: default_locale_manifest,
                    locales: locale_manifests,
                    version: version_manifest,
                })
            })
            .collect::<Vec<_>>();

        // 如果只有一个包，使用它。否则，提示选择要提交的包
        let manifests = match packages.iter_mut().at_most_one() {
            Ok(None) => bail!("在 {} 中未找到有效的包以提交", self.path),
            Ok(Some(manifests)) => manifests,
            Err(_) => &mut Select::new("请选择要提交的包", packages)
                .with_page_size(10)
                .prompt()
                .map_err(handle_inquire_error)?,
        };

        let identifier = &manifests.version.package_identifier;
        let version = &manifests.version.package_version;

        // Reorder the keys in case the manifests weren't created by komac
        manifests.installer.reorder_keys(identifier, version);

        let package_path = get_package_path(identifier, Some(version), None);
        let mut changes = pr_changes()
            .package_identifier(identifier)
            .manifests(manifests)
            .package_path(&package_path)
            .create()?;

        let submit_option = prompt_submit_option(
            &mut changes,
            self.skip_prompt,
            identifier,
            version,
            self.dry_run,
        )?;

        if submit_option == SubmitOption::Exit {
            return Ok(());
        }

        let github = GitHub::new(&token.await?)?;
        let versions = github.get_versions(identifier).await.unwrap_or_default();

        // Create an indeterminate progress bar to show as a pull request is being created
        let pr_progress = ProgressBar::new_spinner().with_message(format!(
            "为 {identifier} 版本 {version} 创建拉取请求",
        ));
        pr_progress.enable_steady_tick(SPINNER_TICK_RATE);

        let pull_request_url = github
            .add_version()
            .identifier(identifier)
            .version(version)
            .versions(&versions)
            .changes(changes)
            .send()
            .await?;

        pr_progress.finish_and_clear();

        println!(
            "{} 创建了一个拉取请求到 {WINGET_PKGS_FULL_NAME}",
            "成功".green()
        );
        println!("{}", pull_request_url.as_str());

        if self.open_pr {
            open::that(pull_request_url.as_str())?;
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
