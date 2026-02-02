use std::{
    collections::BTreeSet,
    mem,
    num::{NonZeroU32, NonZeroUsize},
};

use anstream::println;
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use color_eyre::eyre::Result;
use color_eyre::eyre::ensure;
use indicatif::ProgressBar;
use inquire::CustomType;
use ordinal::Ordinal;
use owo_colors::OwoColorize;
use winget_types::{
    LanguageTag, ManifestType, ManifestVersion, PackageIdentifier, PackageVersion,
    installer::{
        Command, FileExtension, InstallModes, InstallerManifest, InstallerSuccessCode,
        InstallerType, Protocol, UpgradeBehavior,
        switches::{CustomSwitch, InstallerSwitches, SilentSwitch, SilentWithProgressSwitch},
    },
    locale::{
        Author, Copyright, DefaultLocaleManifest, Description, License, Moniker, PackageName,
        Publisher, ShortDescription, Tag,
    },
    url::{
        CopyrightUrl, DecodedUrl, LicenseUrl, PackageUrl, PublisherSupportUrl, PublisherUrl,
        ReleaseNotesUrl,
    },
    version::VersionManifest,
};

use crate::{
    commands::utils::{
        SPINNER_TICK_RATE, SubmitOption, prompt_existing_pull_request, write_changes_to_dir,
    },
    download::{DownloadedFile, Downloader},
    download_file::process_files,
    github::{
        GITHUB_HOST, WINGET_PKGS_FULL_NAME,
        client::GitHub,
        utils::{PackagePath, pull_request::pr_changes},
    },
    manifests::{Manifests, Url},
    prompts::{
        check_prompt, handle_inquire_error,
        list::list_prompt,
        radio_prompt,
        text::{confirm_prompt, optional_prompt, required_prompt},
    },
    terminal::Hyperlinkable,
    token::TokenManager,
};

/// Create a new package from scratch
#[derive(Parser)]
pub struct NewVersion {
    /// The package's unique identifier
    #[arg()]
    package_identifier: Option<PackageIdentifier>,

    /// The package's version
    #[arg(short = 'v', long = "version")]
    package_version: Option<PackageVersion>,

    /// The list of package installers
    #[arg(short, long, num_args = 1.., value_hint = clap::ValueHint::Url)]
    urls: Vec<Url>,

    /// The list of files to use instead of downloading urls
    #[arg(short, long, num_args = 1.., value_hint = clap::ValueHint::FilePath, requires = "urls", value_parser = is_valid_file)]
    files: Vec<Utf8PathBuf>,

    #[arg(long)]
    package_locale: Option<LanguageTag>,

    #[arg(long)]
    publisher: Option<Publisher>,

    #[arg(long, value_hint = clap::ValueHint::Url)]
    publisher_url: Option<PublisherUrl>,

    #[arg(long, value_hint = clap::ValueHint::Url)]
    publisher_support_url: Option<PublisherSupportUrl>,

    #[arg(long)]
    package_name: Option<PackageName>,

    #[arg(long, value_hint = clap::ValueHint::Url)]
    package_url: Option<PackageUrl>,

    #[arg(long)]
    moniker: Option<Moniker>,

    #[arg(long)]
    author: Option<Author>,

    #[arg(long)]
    license: Option<License>,

    #[arg(long, value_hint = clap::ValueHint::Url)]
    license_url: Option<LicenseUrl>,

    #[arg(long)]
    copyright: Option<Copyright>,

    #[arg(long, value_hint = clap::ValueHint::Url)]
    copyright_url: Option<CopyrightUrl>,

    #[arg(long)]
    short_description: Option<ShortDescription>,

    #[arg(long)]
    description: Option<Description>,

    #[arg(long, value_hint = clap::ValueHint::Url)]
    release_notes_url: Option<ReleaseNotesUrl>,

    /// Number of installers to download at the same time
    #[arg(long, default_value_t = NonZeroUsize::new(num_cpus::get()).unwrap())]
    concurrent_downloads: NonZeroUsize,

    /// List of issues that adding this package or version would resolve
    #[arg(long)]
    resolves: Vec<NonZeroU32>,

    /// Automatically submit a pull request
    #[arg(short, long)]
    submit: bool,

    /// Name of external tool that invoked Komac
    #[arg(long, env = "KOMAC_CREATED_WITH")]
    created_with: Option<String>,

    /// URL to external tool that invoked Komac
    #[arg(long, env = "KOMAC_CREATED_WITH_URL", value_hint = clap::ValueHint::Url)]
    created_with_url: Option<DecodedUrl>,

    /// Directory to output the manifests to
    #[arg(short, long, env = "OUTPUT_DIRECTORY", value_hint = clap::ValueHint::DirPath)]
    output: Option<Utf8PathBuf>,

    /// Open pull request link automatically
    #[arg(long, env = "OPEN_PR")]
    open_pr: bool,

    /// Run without prompting or submitting
    #[arg(long, env = "DRY_RUN")]
    dry_run: bool,

    /// Skip checking for existing pull requests
    #[arg(long, env)]
    skip_pr_check: bool,

    /// GitHub personal access token with the `public_repo` scope
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl NewVersion {
    pub async fn run(self) -> Result<()> {
        if !self.files.is_empty() {
            ensure!(
                self.urls.len() == self.files.len(),
                "Number of URLs ({}) must match number of files ({})",
                self.urls.len(),
                self.files.len()
            );
        }

        let token = TokenManager::handle(self.token).await?;
        let github = GitHub::new(&token)?;

        let package_identifier = required_prompt(self.package_identifier, None::<&str>)?;

        let versions = github.get_versions(&package_identifier).await.ok();

        let latest_version = versions.as_ref().and_then(BTreeSet::last);

        if let Some(latest_version) = latest_version {
            println!("Latest version of {package_identifier}: {latest_version}");
        }

        let manifests =
            latest_version.map(|version| github.get_manifests(&package_identifier, version));

        let package_version = required_prompt(self.package_version, None::<&str>)?;

        if !self.skip_pr_check
            && !self.dry_run
            && let Some(pull_request) = github
                .get_existing_pull_request(&package_identifier, &package_version)
                .await?
            && !prompt_existing_pull_request(&package_identifier, &package_version, &pull_request)?
        {
            return Ok(());
        }

        let mut urls = self.urls;
        if urls.is_empty() {
            while urls.len() < 1024 {
                let message = format!("{} Installer URL", Ordinal(urls.len() + 1));
                let url_prompt =
                    CustomType::new(&message).with_error_message("Please enter a valid URL");
                let installer_url = if urls.len() + 1 == 1 {
                    Some(url_prompt.prompt().map_err(handle_inquire_error)?)
                } else {
                    url_prompt
                        .with_help_message("Press ESC if you do not have any more URLs")
                        .prompt_skippable()
                        .map_err(handle_inquire_error)?
                };
                if let Some(url) = installer_url {
                    urls.push(url);
                } else {
                    break;
                }
            }
        }

        let github_values = tokio::spawn({
            let github = github.clone();
            let github_url = urls
                .iter()
                .find(|url| url.host_str() == Some(GITHUB_HOST))
                .cloned();
            async move {
                github_url
                    .map(|url| github.get_all_values_from_url(url.into_inner()))
                    .unwrap_or_default()
                    .await
            }
        });

        let mut files = if self.files.is_empty() {
            let downloader = Downloader::new_with_concurrent(self.concurrent_downloads)?;
            downloader.download(urls.iter().cloned()).await?
        } else {
            self.files
                .iter()
                .zip(urls.iter().cloned())
                .map(|(path, url)| DownloadedFile::from_local(path, url))
                .collect::<Result<Vec<_>>>()?
        };

        let mut download_results = process_files(&mut files).await?;

        let mut installers = Vec::new();
        for analyser in &mut download_results.values_mut() {
            let mut silent = None;
            let mut silent_with_progress = None;
            let mut custom = None;
            if analyser.installers.iter().any(|installer| {
                installer.r#type == Some(InstallerType::Exe) && installer.switches.is_empty()
            }) {
                if confirm_prompt(&format!("Is {} a portable exe?", analyser.file_name))? {
                    for installer in &mut analyser.installers {
                        installer.r#type = Some(InstallerType::Portable);
                    }
                }
                silent = Some(required_prompt::<SilentSwitch, &str>(None, None)?);
                silent_with_progress = Some(required_prompt::<SilentWithProgressSwitch, &str>(
                    None, None,
                )?);
            }
            if analyser
                .installers
                .iter()
                .any(|installer| installer.r#type == Some(InstallerType::Portable))
            {
                custom = optional_prompt::<CustomSwitch, &str>(None, None)?;
            }
            if let Some(zip) = &mut analyser.zip {
                zip.prompt()?;
                for (analyser_installer, zip_installer) in
                    analyser.installers.iter_mut().zip(zip.installers.iter())
                {
                    analyser_installer.nested_installer_type = zip_installer.nested_installer_type;
                    analyser_installer.nested_installer_files =
                        zip_installer.nested_installer_files.clone();
                }
            }
            let switches = InstallerSwitches::builder()
                .maybe_silent(silent)
                .maybe_silent_with_progress(silent_with_progress)
                .maybe_custom(custom)
                .build();
            let mut analyser_installers = mem::take(&mut analyser.installers);
            for installer in &mut analyser_installers {
                if !switches.is_empty() {
                    installer.switches = switches.clone();
                }
            }
            installers.extend(analyser_installers);
        }

        let default_locale = required_prompt(self.package_locale, None::<&str>)?;
        let mut installer_manifest = InstallerManifest {
            package_identifier: package_identifier.clone(),
            package_version: package_version.clone(),
            install_modes: if installers
                .iter()
                .any(|installer| installer.r#type == Some(InstallerType::Inno))
            {
                InstallModes::all()
            } else {
                check_prompt::<InstallModes>()?
            },
            success_codes: list_prompt::<InstallerSuccessCode>()?,
            upgrade_behavior: Some(radio_prompt::<UpgradeBehavior>()?),
            commands: list_prompt::<Command>()?,
            protocols: list_prompt::<Protocol>()?,
            file_extensions: if installers
                .iter()
                .all(|installer| installer.file_extensions.is_empty())
            {
                list_prompt::<FileExtension>()?
            } else {
                BTreeSet::new()
            },
            installers,
            manifest_type: ManifestType::Installer,
            ..InstallerManifest::default()
        };

        let mut github_values = match github_values.await? {
            Some(future) => Some(future?),
            None => None,
        };

        let default_locale_manifest = DefaultLocaleManifest {
            package_identifier: package_identifier.clone(),
            package_version: package_version.clone(),
            package_locale: default_locale.clone(),
            publisher: required_prompt(
                self.publisher,
                download_results
                    .values()
                    .find(|analyser| analyser.publisher.is_some())
                    .and_then(|analyser| analyser.publisher.as_ref()),
            )?,
            publisher_url: optional_prompt(
                self.publisher_url,
                github_values.as_ref().map(|values| &values.publisher_url),
            )?,
            publisher_support_url: optional_prompt(
                self.publisher_support_url,
                github_values
                    .as_ref()
                    .and_then(|values| values.issues_url.as_ref()),
            )?,
            author: optional_prompt(self.author, None::<&str>)?,
            package_name: required_prompt(
                self.package_name,
                download_results
                    .values()
                    .find(|analyser| analyser.package_name.is_some())
                    .and_then(|analyser| analyser.package_name.as_ref()),
            )?,
            package_url: optional_prompt(
                self.package_url,
                github_values.as_ref().map(|values| &values.package_url),
            )?,
            license: required_prompt(
                self.license,
                github_values
                    .as_ref()
                    .and_then(|values| values.license.as_ref()),
            )?,
            license_url: optional_prompt(
                self.license_url,
                github_values
                    .as_ref()
                    .and_then(|values| values.license_url.as_ref()),
            )?,
            copyright: optional_prompt(
                self.copyright,
                download_results
                    .values()
                    .find(|analyser| analyser.copyright.is_some())
                    .and_then(|analyser| analyser.copyright.as_ref()),
            )?,
            copyright_url: optional_prompt(self.copyright_url, None::<&str>)?,
            short_description: required_prompt(
                self.short_description,
                github_values
                    .as_ref()
                    .and_then(|values| values.description.as_ref()),
            )?,
            description: optional_prompt(self.description, None::<&str>)?,
            moniker: optional_prompt(self.moniker, None::<&str>)?,
            tags: match github_values
                .as_mut()
                .map(|values| mem::take(&mut values.topics))
            {
                Some(topics) => topics,
                None => list_prompt::<Tag>()?,
            },
            release_notes: github_values
                .as_mut()
                .and_then(|values| values.release_notes.take()),
            release_notes_url: optional_prompt(
                self.release_notes_url,
                github_values
                    .as_ref()
                    .and_then(|values| values.release_notes_url.as_ref()),
            )?,
            manifest_type: ManifestType::DefaultLocale,
            ..DefaultLocaleManifest::default()
        };

        installer_manifest
            .installers
            .iter_mut()
            .flat_map(|installer| &mut installer.apps_and_features_entries)
            .for_each(|entry| entry.deduplicate(&default_locale_manifest));

        installer_manifest.optimize();

        let version_manifest = VersionManifest {
            package_identifier: package_identifier.clone(),
            package_version: package_version.clone(),
            default_locale,
            manifest_type: ManifestType::Version,
            manifest_version: ManifestVersion::default(),
        };

        let manifests = match manifests {
            Some(manifests) => Some(manifests.await?),
            None => None,
        };

        let manifests = Manifests {
            installer: installer_manifest,
            default_locale: default_locale_manifest,
            locales: manifests
                .map(|manifests| manifests.locales)
                .unwrap_or_default(),
            version: version_manifest,
        };

        let package_path = PackagePath::new(&package_identifier, Some(&package_version), None);
        let mut changes = pr_changes()
            .package_identifier(&package_identifier)
            .manifests(&manifests)
            .package_path(&package_path)
            .maybe_created_with(self.created_with.as_deref())
            .create()?;

        if let Some(output) = self.output.map(|out| out.join(package_path.as_str())) {
            write_changes_to_dir(&changes, output.as_path()).await?;
            println!(
                "{} written all manifest files to {output}",
                "Successfully".green()
            );
        }

        let submit_option = SubmitOption::prompt(
            &mut changes,
            &package_identifier,
            &package_version,
            self.submit,
            self.dry_run,
        )?;

        if submit_option.is_exit() {
            return Ok(());
        }

        // Create an indeterminate progress bar to show as a pull request is being created
        let pr_progress = ProgressBar::new_spinner().with_message(format!(
            "Creating a pull request for {package_identifier} {package_version}"
        ));
        pr_progress.enable_steady_tick(SPINNER_TICK_RATE);

        let pull_request_url = github
            .add_version()
            .identifier(&package_identifier)
            .version(&package_version)
            .maybe_versions(versions.as_ref())
            .changes(changes)
            .issue_resolves(&self.resolves)
            .maybe_created_with(self.created_with.as_deref())
            .maybe_created_with_url(self.created_with_url.as_ref())
            .send()
            .await?;

        pr_progress.finish_and_clear();

        println!(
            "{} created a {} to {WINGET_PKGS_FULL_NAME}",
            "Successfully".green(),
            "pull request".hyperlink(&pull_request_url)
        );

        if self.open_pr {
            open::that(pull_request_url.as_str())?;
        }

        Ok(())
    }
}

fn is_valid_file(path: &str) -> Result<Utf8PathBuf> {
    let path = Utf8Path::new(path);
    ensure!(path.exists(), "{path} does not exist");
    ensure!(path.is_file(), "{path} is not a file");
    Ok(path.to_path_buf())
}
