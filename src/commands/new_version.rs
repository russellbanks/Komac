use std::collections::BTreeSet;
use std::mem;
use std::num::{NonZeroU8, NonZeroU32};

use crate::commands::utils::{
    SPINNER_TICK_RATE, SubmitOption, prompt_existing_pull_request, prompt_submit_option,
    write_changes_to_dir,
};
use crate::credential::{get_default_headers, handle_token};
use crate::download_file::{download_urls, process_files};
use crate::github::github_client::{GITHUB_HOST, GitHub, WINGET_PKGS_FULL_NAME};
use crate::github::utils::get_package_path;
use crate::github::utils::pull_request::pr_changes;
use crate::manifests::Manifests;
use crate::manifests::default_locale_manifest::DefaultLocaleManifest;
use crate::manifests::installer_manifest::{InstallerManifest, InstallerSwitches, UpgradeBehavior};
use crate::manifests::version_manifest::VersionManifest;
use crate::prompts::list::list_prompt;
use crate::prompts::text::{confirm_prompt, optional_prompt, required_prompt};
use crate::prompts::{check_prompt, handle_inquire_error, radio_prompt};
use crate::types::author::Author;
use crate::types::command::Command;
use crate::types::copyright::Copyright;
use crate::types::custom_switch::CustomSwitch;
use crate::types::description::Description;
use crate::types::file_extension::FileExtension;
use crate::types::install_modes::InstallModes;
use crate::types::installer_success_code::InstallerSuccessCode;
use crate::types::installer_type::InstallerType;
use crate::types::language_tag::LanguageTag;
use crate::types::license::License;
use crate::types::manifest_type::ManifestType;
use crate::types::manifest_version::ManifestVersion;
use crate::types::moniker::Moniker;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_name::PackageName;
use crate::types::package_version::PackageVersion;
use crate::types::protocol::Protocol;
use crate::types::publisher::Publisher;
use crate::types::short_description::ShortDescription;
use crate::types::silent_switch::SilentSwitch;
use crate::types::silent_with_progress_switch::SilentWithProgressSwitch;
use crate::types::tag::Tag;
use crate::types::urls::copyright_url::CopyrightUrl;
use crate::types::urls::license_url::LicenseUrl;
use crate::types::urls::package_url::PackageUrl;
use crate::types::urls::publisher_support_url::PublisherSupportUrl;
use crate::types::urls::publisher_url::PublisherUrl;
use crate::types::urls::release_notes_url::ReleaseNotesUrl;
use crate::types::urls::url::DecodedUrl;
use anstream::println;
use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::eyre::Result;
use indicatif::ProgressBar;
use inquire::CustomType;
use ordinal_trait::Ordinal;
use owo_colors::OwoColorize;
use reqwest::Client;

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
    urls: Vec<DecodedUrl>,

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
    #[arg(long, default_value_t = NonZeroU8::new(2).unwrap())]
    concurrent_downloads: NonZeroU8,

    /// List of issues that adding this package or version would resolve
    #[arg(long)]
    resolves: Option<Vec<NonZeroU32>>,

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
        let token = handle_token(self.token.as_deref()).await?;
        let github = GitHub::new(&token)?;
        let client = Client::builder()
            .default_headers(get_default_headers(None))
            .build()?;

        let package_identifier = required_prompt(self.package_identifier)?;

        let versions = github.get_versions(&package_identifier).await.ok();

        let latest_version = versions.as_ref().and_then(BTreeSet::last);

        if let Some(latest_version) = latest_version {
            println!("Latest version of {package_identifier}: {latest_version}");
        }

        let manifests =
            latest_version.map(|version| github.get_manifests(&package_identifier, version));

        let package_version = required_prompt(self.package_version)?;

        if let Some(pull_request) = github
            .get_existing_pull_request(&package_identifier, &package_version)
            .await?
        {
            if !(self.skip_pr_check || self.dry_run)
                && !prompt_existing_pull_request(
                    &package_identifier,
                    &package_version,
                    &pull_request,
                )?
            {
                return Ok(());
            }
        }

        let mut urls = self.urls;
        if urls.is_empty() {
            while urls.len() < 1024 {
                let message = format!("{} Installer URL", (urls.len() + 1).to_number());
                let url_prompt = CustomType::<DecodedUrl>::new(&message)
                    .with_error_message("Please enter a valid URL");
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

        let github_values = urls
            .iter()
            .find(|url| url.host_str() == Some(GITHUB_HOST))
            .and_then(|url| github.get_all_values_from_url(url));

        let mut files = download_urls(&client, urls, self.concurrent_downloads).await?;
        let mut download_results = process_files(&mut files).await?;

        let mut installers = Vec::new();
        for analyser in &mut download_results.values_mut() {
            let mut installer_switches = InstallerSwitches::default();
            if analyser
                .installers
                .iter()
                .any(|installer| installer.r#type == Some(InstallerType::Exe))
            {
                if confirm_prompt(&format!("Is {} a portable exe?", analyser.file_name))? {
                    for installer in &mut analyser.installers {
                        installer.r#type = Some(InstallerType::Portable);
                    }
                }
                installer_switches.silent = Some(required_prompt::<SilentSwitch>(None)?);
                installer_switches.silent_with_progress =
                    Some(required_prompt::<SilentWithProgressSwitch>(None)?);
            }
            if analyser
                .installers
                .iter()
                .any(|installer| installer.r#type == Some(InstallerType::Portable))
            {
                installer_switches.custom = optional_prompt::<CustomSwitch>(None)?;
            }
            if let Some(zip) = &mut analyser.zip {
                zip.prompt()?;
            }
            let mut analyser_installers = mem::take(&mut analyser.installers);
            for installer in &mut analyser_installers {
                if installer_switches.is_any_some() {
                    installer.switches = Some(installer_switches.clone());
                }
            }
            installers.extend(analyser_installers);
        }
        let default_locale = required_prompt(self.package_locale)?;
        let manifests = match manifests {
            Some(manifests) => Some(manifests.await?),
            None => None,
        };
        let mut installer_manifest = InstallerManifest {
            package_identifier: package_identifier.clone(),
            package_version: package_version.clone(),
            install_modes: if installers
                .iter()
                .any(|installer| installer.r#type == Some(InstallerType::Inno))
            {
                Some(InstallModes::all())
            } else {
                check_prompt::<InstallModes>()?
            },
            success_codes: list_prompt::<InstallerSuccessCode>()?,
            upgrade_behavior: Some(radio_prompt::<UpgradeBehavior>()?),
            commands: list_prompt::<Command>()?,
            protocols: list_prompt::<Protocol>()?,
            file_extensions: if installers
                .iter()
                .all(|installer| installer.file_extensions.is_none())
            {
                list_prompt::<FileExtension>()?
            } else {
                None
            },
            installers,
            manifest_type: ManifestType::Installer,
            ..InstallerManifest::default()
        };

        let mut github_values = match github_values {
            Some(future) => Some(future.await?),
            None => None,
        };
        let default_locale_manifest = DefaultLocaleManifest {
            package_identifier: package_identifier.clone(),
            package_version: package_version.clone(),
            package_locale: default_locale.clone(),
            publisher: match download_results
                .values_mut()
                .find(|analyser| analyser.publisher.is_some())
                .and_then(|analyser| analyser.publisher.take())
            {
                Some(publisher) => publisher,
                None => required_prompt(self.publisher)?,
            },
            publisher_url: optional_prompt(self.publisher_url)?,
            publisher_support_url: optional_prompt(self.publisher_support_url)?,
            author: optional_prompt(self.author)?,
            package_name: match download_results
                .values_mut()
                .find(|analyser| analyser.package_name.is_some())
                .and_then(|analyser| analyser.package_name.take())
            {
                Some(package_name) => package_name,
                None => required_prompt(self.package_name)?,
            },
            package_url: optional_prompt(self.package_url)?,
            license: match github_values
                .as_mut()
                .and_then(|values| values.license.take())
            {
                Some(license) => license,
                None => required_prompt(self.license)?,
            },
            license_url: optional_prompt(self.license_url)?,
            copyright: match download_results
                .values_mut()
                .find(|analyser| analyser.copyright.is_some())
                .and_then(|analyser| analyser.copyright.take())
            {
                Some(copyright) => Some(copyright),
                None => optional_prompt(self.copyright)?,
            },
            copyright_url: optional_prompt(self.copyright_url)?,
            short_description: required_prompt(self.short_description)?,
            description: optional_prompt(self.description)?,
            moniker: optional_prompt(self.moniker)?,
            tags: match github_values
                .as_mut()
                .and_then(|values| values.topics.take())
            {
                Some(topics) => Some(topics),
                None => list_prompt::<Tag>()?,
            },
            release_notes_url: optional_prompt(self.release_notes_url)?,
            manifest_type: ManifestType::DefaultLocale,
            ..DefaultLocaleManifest::default()
        };

        installer_manifest
            .installers
            .iter_mut()
            .filter_map(|installer| installer.apps_and_features_entries.as_mut())
            .flatten()
            .for_each(|entry| entry.deduplicate(&package_version, &default_locale_manifest));

        installer_manifest.reorder_keys(&package_identifier, &package_version);

        let version_manifest = VersionManifest {
            package_identifier: package_identifier.clone(),
            package_version: package_version.clone(),
            default_locale,
            manifest_type: ManifestType::Version,
            manifest_version: ManifestVersion::default(),
        };

        let manifests = Manifests {
            installer: installer_manifest,
            default_locale: default_locale_manifest,
            locales: manifests
                .map(|manifests| manifests.locales)
                .unwrap_or_default(),
            version: version_manifest,
        };

        let package_path = get_package_path(&package_identifier, Some(&package_version), None);
        let mut changes = pr_changes()
            .package_identifier(&package_identifier)
            .manifests(&manifests)
            .package_path(&package_path)
            .maybe_created_with(self.created_with.as_deref())
            .create()?;

        if let Some(output) = self.output.map(|out| out.join(package_path)) {
            write_changes_to_dir(&changes, output.as_path()).await?;
            println!(
                "{} written all manifest files to {output}",
                "Successfully".green()
            );
        }

        let submit_option = prompt_submit_option(
            &mut changes,
            self.submit,
            &package_identifier,
            &package_version,
            self.dry_run,
        )?;

        if submit_option == SubmitOption::Exit {
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
            .maybe_issue_resolves(self.resolves)
            .maybe_created_with(self.created_with)
            .maybe_created_with_url(self.created_with_url)
            .send()
            .await?;

        pr_progress.finish_and_clear();

        println!(
            "{} created a pull request to {WINGET_PKGS_FULL_NAME}",
            "Successfully".green()
        );
        println!("{}", pull_request_url.as_str());

        if self.open_pr {
            open::that(pull_request_url.as_str())?;
        }

        Ok(())
    }
}
