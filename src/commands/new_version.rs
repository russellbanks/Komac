use std::{
    collections::BTreeSet,
    mem,
    num::{NonZeroU32, NonZeroUsize},
    path::PathBuf,
};

use anstream::println;
use clap::Parser;
use color_eyre::eyre::Result;
use indicatif::ProgressBar;
use inquire::CustomType;
use secrecy::SecretString;
use winget_types::{
    LanguageTag, PackageIdentifier, PackageVersion, VersionManifest,
    installer::{
        Command, FileExtension, InstallModes, InstallerManifest, InstallerSuccessCode,
        InstallerType, Protocol, Switches, UpgradeBehavior,
        switches::{CustomSwitch, SilentSwitch, SilentWithProgressSwitch},
    },
    locale::{
        Author, Copyright, DefaultLocaleManifest, Description, License, Moniker, PackageName,
        Publisher, ShortDescription, Tag,
    },
    url::{
        CopyrightUrl, DecodedUrl, LicenseUrl, PackageUrl, PublisherSupportUrl, PublisherUrl,
        ReleaseNotesUrl,
    },
};

use crate::{
    commands::utils::{SPINNER_TICK_RATE, SubmitOption},
    download::Downloader,
    github::{
        GITHUB_HOST,
        client::GitHub,
        utils::{PackagePath, pull_request::Change},
    },
    i18n::fl,
    manifests::{Manifests, Url, print_changes},
    prompts::{
        check_prompt, handle_inquire_error,
        list::list_prompt,
        radio_prompt,
        text::{confirm_prompt, optional_prompt, required_prompt},
    },
    terminal::success,
    token::TokenManager,
};

#[expect(clippy::struct_excessive_bools, reason = "CLI flags")]
#[derive(Parser)]
#[command(about = fl!("command-new"))]
pub struct NewVersion {
    #[arg(value_name = "PACKAGE_IDENTIFIER", help = fl!("arg-identifier"))]
    identifier: Option<PackageIdentifier>,

    #[arg(short = 'v', long = "version", help = fl!("arg-version"))]
    version: Option<PackageVersion>,

    #[arg(short, long, num_args = 1.., value_hint = clap::ValueHint::Url, help = fl!("arg-urls"))]
    urls: Vec<Url>,

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

    #[arg(
        long,
        default_value_t = std::thread::available_parallelism().unwrap_or(NonZeroUsize::MIN),
        help = fl!("arg-concurrent-downloads"),
    )]
    concurrent_downloads: NonZeroUsize,

    #[arg(long, help = fl!("arg-new-resolves"))]
    resolves: Vec<NonZeroU32>,

    #[arg(short, long, help = fl!("arg-submit"))]
    submit: bool,

    #[arg(long, conflicts_with_all = ["submit", "dry_run"], help = fl!("arg-exit"))]
    exit: bool,

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

    #[arg(long, env = "DRY_RUN", help = fl!("arg-new-dry-run"))]
    dry_run: bool,

    #[arg(long, env, help = fl!("arg-skip-pr-check"))]
    skip_pr_check: bool,

    #[arg(short, long, env = "GITHUB_TOKEN", hide_env_values = true, help = fl!("arg-token"))]
    token: Option<SecretString>,
}

impl NewVersion {
    pub async fn run(self) -> Result<()> {
        let token_manager = TokenManager::handle(self.token).await?;
        let github = GitHub::new(token_manager)?;

        let identifier = required_prompt(self.identifier, None::<&str>)?;

        let package = github.get_package(&identifier).await?;

        if let Some(latest_version) = package.latest_version() {
            println!(
                "{}",
                fl!(
                    "latest-version",
                    identifier = identifier.as_str(),
                    version = latest_version.to_string()
                )
            );
        }

        let version = required_prompt(self.version, None::<&str>)?;

        let mut package = package.into_versioned(&version, &github).await?;
        if !self.skip_pr_check && !self.dry_run && !package.prompt_existing_pr()? {
            return Ok(());
        }

        let mut urls = self.urls;
        if urls.is_empty() {
            let invalid_message = fl!("prompt-installer-url", "invalid");
            let help_message = fl!("prompt-installer-url", "help");
            while urls.len() < 1024 {
                let position = urls.len() + 1;
                let message = fl!("prompt-installer-url", position = position);
                let url_prompt = CustomType::new(&message).with_error_message(&invalid_message);
                let installer_url = if urls.is_empty() {
                    Some(url_prompt.prompt().map_err(handle_inquire_error)?)
                } else {
                    url_prompt
                        .with_help_message(&help_message)
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

        let downloader = Downloader::new_with_concurrent(self.concurrent_downloads)?;
        let mut files = downloader.download(urls.iter().cloned()).await?;
        let mut download_results = files.analyze().await?;

        let mut installers = Vec::new();
        for analyzer in &mut download_results.values_mut() {
            let mut silent = None;
            let mut silent_with_progress = None;
            let mut custom = None;
            if analyzer
                .installers
                .iter()
                .any(|installer| installer.r#type == Some(InstallerType::Exe))
            {
                if confirm_prompt(&fl!(
                    "prompt-portable-exe",
                    file = analyzer.file_name.as_str()
                ))? {
                    for installer in &mut analyzer.installers {
                        installer.r#type = Some(InstallerType::Portable);
                    }
                }
                silent = Some(required_prompt::<SilentSwitch, &str>(None, None)?);
                silent_with_progress = Some(required_prompt::<SilentWithProgressSwitch, &str>(
                    None, None,
                )?);
            }
            if analyzer
                .installers
                .iter()
                .any(|installer| installer.r#type == Some(InstallerType::Portable))
            {
                custom = optional_prompt::<CustomSwitch, &str>(None, None)?;
            }
            if let Some(zip) = &mut analyzer.zip {
                zip.prompt()?;
                for (analyzer_installer, zip_installer) in
                    analyzer.installers.iter_mut().zip(zip.installers.iter())
                {
                    analyzer_installer.nested_installer_type = zip_installer.nested_installer_type;
                    analyzer_installer
                        .nested_installer_files
                        .clone_from(&zip_installer.nested_installer_files);
                }
            }
            let switches = Switches::builder()
                .maybe_silent(silent)
                .maybe_silent_with_progress(silent_with_progress)
                .maybe_custom(custom)
                .build();
            let mut analyzer_installers = mem::take(&mut analyzer.installers);
            for installer in &mut analyzer_installers {
                if !switches.is_empty() {
                    installer.switches = switches.clone();
                }
            }
            installers.extend(analyzer_installers);
        }

        let default_locale = required_prompt(self.package_locale, Some("en-US"))?;
        let mut installer_manifest = InstallerManifest {
            package_identifier: identifier.clone(),
            package_version: version.clone(),
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
            ..InstallerManifest::default()
        };

        let mut github_values = match github_values.await? {
            Some(future) => Some(future?),
            None => None,
        };

        let default_locale_manifest = DefaultLocaleManifest {
            package_identifier: identifier.clone(),
            package_version: version.clone(),
            package_locale: default_locale.clone(),
            publisher: required_prompt(
                self.publisher,
                download_results
                    .values()
                    .find(|analyzer| analyzer.publisher.is_some())
                    .and_then(|analyzer| analyzer.publisher.as_ref())
                    .or_else(|| {
                        github_values
                            .as_ref()
                            .and_then(|values| values.publisher.as_ref())
                    }),
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
                    .find(|analyzer| analyzer.package_name.is_some())
                    .and_then(|analyzer| analyzer.package_name.as_ref()),
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
                    .find(|analyzer| analyzer.copyright.is_some())
                    .and_then(|analyzer| analyzer.copyright.as_ref()),
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
            ..DefaultLocaleManifest::default()
        };

        installer_manifest
            .installers
            .iter_mut()
            .flat_map(|installer| &mut installer.apps_and_features_entries)
            .for_each(|entry| entry.deduplicate(&default_locale_manifest));

        installer_manifest.optimize();

        let manifests = Manifests {
            installer: installer_manifest,
            default_locale: default_locale_manifest,
            locales: package
                .manifests
                .take()
                .map(|manifests| manifests.locales)
                .unwrap_or_default(),
            version: VersionManifest::new(identifier.clone(), version.clone(), default_locale),
        };

        let mut changes = manifests.create(&identifier, &version, self.created_with.as_deref());

        if self.dry_run {
            print_changes(changes.iter().map(Change::manifest));
            return Ok(());
        }

        let submit_option =
            SubmitOption::prompt(&mut changes, &identifier, &version, self.submit, self.exit)?;

        let package_path = PackagePath::new(&identifier, Some(&version), None);
        if let Some(output) = self.output.map(|out| out.join(package_path.as_str())) {
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
            identifier = identifier.as_str(),
            version = version.to_string()
        ));
        pr_progress.enable_steady_tick(SPINNER_TICK_RATE);

        let pull_request = github
            .add_version()
            .identifier(&identifier)
            .version(&version)
            .versions(package.versions())
            .changes(changes)
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
}
