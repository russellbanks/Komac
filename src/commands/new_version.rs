use crate::credential::{get_default_headers, handle_token};
use crate::default_locale_manifest::DefaultLocaleManifest;
use crate::download_file::{download_urls, process_files};
use crate::github::github_client::GitHub;
use crate::github::github_utils::{get_full_package_path, get_package_path};
use crate::installer_manifest::{
    InstallModes, Installer, InstallerManifest, InstallerSwitches, InstallerType, UpgradeBehavior,
};
use crate::locale_manifest::LocaleManifest;
use crate::manifest::{build_manifest_string, print_changes, Manifest};
use crate::prompts::list_prompt::list_prompt;
use crate::prompts::multi_prompt::{check_prompt, radio_prompt};
use crate::prompts::prompt::{optional_prompt, required_prompt};
use crate::types::author::Author;
use crate::types::command::Command;
use crate::types::copyright::Copyright;
use crate::types::custom_switch::CustomSwitch;
use crate::types::description::Description;
use crate::types::file_extension::FileExtension;
use crate::types::installer_success_code::InstallerSuccessCode;
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
use crate::types::urls::publisher_url::PublisherUrl;
use crate::types::urls::release_notes_url::ReleaseNotesUrl;
use crate::version_manifest::VersionManifest;
use clap::Parser;
use color_eyre::eyre::Result;
use futures_util::{stream, StreamExt, TryStreamExt};
use indicatif::MultiProgress;
use inquire::{Confirm, CustomType};
use percent_encoding::percent_decode_str;
use reqwest::Client;
use std::collections::BTreeSet;
use std::num::NonZeroU8;
use url::Url;

#[derive(Parser)]
pub struct New {
    #[arg(short = 'i', long = "identifier")]
    package_identifier: Option<PackageIdentifier>,

    #[arg(short = 'v', long = "version")]
    package_version: Option<PackageVersion>,

    #[arg(short, long, num_args=1..)]
    urls: Vec<Url>,

    #[arg(long)]
    package_locale: Option<LanguageTag>,

    #[arg(long)]
    publisher: Option<Publisher>,

    #[arg(long)]
    publisher_url: Option<PublisherUrl>,

    #[arg(long)]
    package_name: Option<PackageName>,

    #[arg(long)]
    package_url: Option<PackageUrl>,

    #[arg(long)]
    moniker: Option<Moniker>,

    #[arg(long)]
    author: Option<Author>,

    #[arg(long)]
    license: Option<License>,

    #[arg(long)]
    license_url: Option<LicenseUrl>,

    #[arg(long)]
    copyright: Option<Copyright>,

    #[arg(long)]
    copyright_url: Option<CopyrightUrl>,

    #[arg(long)]
    short_description: Option<ShortDescription>,

    #[arg(long)]
    description: Option<Description>,

    #[arg(long)]
    release_notes_url: Option<ReleaseNotesUrl>,

    // Number of installers to download at the same time
    #[arg(long, default_value_t = NonZeroU8::new(2).unwrap())]
    concurrent_downloads: NonZeroU8,

    #[arg(short, long)]
    submit: bool,

    /// GitHub personal access token with the public_repo scope
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl New {
    pub async fn run(self) -> Result<()> {
        let token = handle_token(self.token).await?;
        let github = GitHub::new(token)?;
        let client = Client::builder()
            .default_headers(get_default_headers(None))
            .build()?;

        let package_identifier = required_prompt(self.package_identifier)?;

        let versions = github
            .get_versions(&get_package_path(&package_identifier))
            .await
            .ok();

        let latest_version = versions.as_ref().and_then(|versions| versions.iter().max());

        if let Some(latest_version) = latest_version {
            println!("Latest version of {package_identifier}: {latest_version}");
        }

        let manifests =
            latest_version.map(|version| github.get_manifests(&package_identifier, version));

        let package_version = required_prompt(self.package_version)?;

        let mut urls = self.urls;
        if urls.is_empty() {
            let mut count: u16 = 1;
            while urls.len() < 1024 {
                let suffix = match count % 100 {
                    11..=13 => "th",
                    _ => match count % 10 {
                        1 => "st",
                        2 => "nd",
                        3 => "rd",
                        _ => "th",
                    },
                };
                let message = format!("{count}{suffix} Installer URL");
                let url_prompt =
                    CustomType::<Url>::new(&message).with_error_message("Please enter a valid URL");
                let installer_url = if count == 1 {
                    Some(url_prompt.prompt()?)
                } else {
                    url_prompt
                        .with_help_message("Press ESC if you do not have any more URLs")
                        .prompt_skippable()?
                };
                if let Some(url) = installer_url {
                    count += 1;
                    urls.push(url)
                } else {
                    break;
                }
            }
        }

        let multi_progress = MultiProgress::new();
        let files = stream::iter(download_urls(&client, &urls, &multi_progress))
            .buffer_unordered(self.concurrent_downloads.get() as usize)
            .try_collect::<Vec<_>>()
            .await?;
        multi_progress.clear()?;
        let download_results = process_files(files).await?;
        let mut installers = BTreeSet::new();
        for (url, mut data) in download_results {
            if data.installer_type == InstallerType::Exe
                && Confirm::new(&format!("Is {} a portable exe?", data.file_name)).prompt()?
            {
                data.installer_type = InstallerType::Portable;
            }
            let mut installer_switches = InstallerSwitches {
                ..InstallerSwitches::default()
            };
            if data.installer_type == InstallerType::Exe {
                installer_switches.silent = optional_prompt::<SilentSwitch>(None)?;
                installer_switches.silent_with_progress =
                    optional_prompt::<SilentWithProgressSwitch>(None)?;
            }
            if data.installer_type != InstallerType::Portable {
                installer_switches.custom = optional_prompt::<CustomSwitch>(None)?
            }
            installers.insert(Installer {
                platform: data
                    .msix
                    .as_ref()
                    .map(|msix| BTreeSet::from([msix.target_device_family])),
                architecture: data.architecture,
                installer_url: Url::parse(
                    &percent_decode_str(url).decode_utf8().unwrap_or_default(),
                )?,
                installer_sha_256: data.installer_sha_256,
                installer_switches: if installer_switches.are_all_none() {
                    None
                } else {
                    Some(installer_switches)
                },
                ..Installer::default()
            });
        }
        let default_locale = required_prompt(self.package_locale)?;
        let manifests = match manifests {
            Some(manifests) => Some(manifests.await?),
            None => None,
        };
        let installer_manifest = InstallerManifest {
            package_identifier: package_identifier.clone(),
            package_version: package_version.clone(),
            install_modes: check_prompt::<InstallModes>()?,
            installer_success_codes: list_prompt::<InstallerSuccessCode>()?,
            upgrade_behavior: Some(radio_prompt::<UpgradeBehavior>()?),
            commands: list_prompt::<Command>()?,
            protocols: list_prompt::<Protocol>()?,
            file_extensions: list_prompt::<FileExtension>()?,
            installers,
            manifest_type: ManifestType::Installer,
            manifest_version: ManifestVersion::default(),
            ..InstallerManifest::default()
        };
        let default_locale_manifest = DefaultLocaleManifest {
            package_identifier: package_identifier.clone(),
            package_version: package_version.clone(),
            package_locale: default_locale.clone(),
            publisher: required_prompt(self.publisher)?,
            publisher_url: optional_prompt(self.publisher_url)?,
            author: optional_prompt(self.author)?,
            package_name: required_prompt(self.package_name)?,
            package_url: optional_prompt(self.package_url)?,
            license: required_prompt(self.license)?,
            license_url: optional_prompt(self.license_url)?,
            copyright: optional_prompt(self.copyright)?,
            copyright_url: optional_prompt(self.copyright_url)?,
            short_description: required_prompt(self.short_description)?,
            description: optional_prompt(self.description)?,
            moniker: optional_prompt(self.moniker)?,
            tags: list_prompt::<Tag>()?,
            release_notes_url: optional_prompt(self.release_notes_url)?,
            manifest_type: ManifestType::DefaultLocale,
            manifest_version: ManifestVersion::default(),
            ..DefaultLocaleManifest::default()
        };
        let version_manifest = VersionManifest {
            package_identifier: package_identifier.clone(),
            package_version: package_version.clone(),
            default_locale,
            manifest_type: ManifestType::Version,
            manifest_version: ManifestVersion::default(),
        };

        let changes = {
            let full_package_path = get_full_package_path(&package_identifier, &package_version);
            let mut path_content_map = Vec::new();
            path_content_map.push((
                format!("{full_package_path}/{}.installer.yaml", package_identifier),
                build_manifest_string(Manifest::Installer(&installer_manifest))?,
            ));
            path_content_map.push((
                format!(
                    "{full_package_path}/{}.locale.{}.yaml",
                    package_identifier, version_manifest.default_locale
                ),
                build_manifest_string(Manifest::DefaultLocale(&default_locale_manifest))?,
            ));
            if let Some(locale_manifests) = manifests.map(|manifests| manifests.locale_manifests) {
                locale_manifests
                    .into_iter()
                    .map(|locale_manifest| LocaleManifest {
                        manifest_version: ManifestVersion::default(),
                        ..locale_manifest
                    })
                    .for_each(|locale_manifest| {
                        if let Ok(yaml) = build_manifest_string(Manifest::Locale(&locale_manifest))
                        {
                            path_content_map.push((
                                format!(
                                    "{full_package_path}/{}.locale.{}.yaml",
                                    package_identifier, locale_manifest.package_locale
                                ),
                                yaml,
                            ));
                        }
                    });
            }
            path_content_map.push((
                format!("{full_package_path}/{}.yaml", package_identifier),
                build_manifest_string(Manifest::Version(&version_manifest))?,
            ));
            path_content_map
        };

        print_changes(&changes);

        Ok(())
    }
}
