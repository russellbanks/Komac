use const_format::formatc;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::BTreeSet;
use std::mem;
use url::Url;

use crate::github::github_client::GitHubValues;
use crate::manifests::ManifestTrait;
use crate::types::author::Author;
use crate::types::copyright::Copyright;
use crate::types::description::Description;
use crate::types::installation_notes::InstallationNotes;
use crate::types::language_tag::LanguageTag;
use crate::types::license::License;
use crate::types::manifest_type::ManifestType;
use crate::types::manifest_version::ManifestVersion;
use crate::types::moniker::Moniker;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_name::PackageName;
use crate::types::package_version::PackageVersion;
use crate::types::publisher::Publisher;
use crate::types::release_notes::ReleaseNotes;
use crate::types::sha_256::Sha256String;
use crate::types::short_description::ShortDescription;
use crate::types::tag::Tag;
use crate::types::urls::copyright_url::CopyrightUrl;
use crate::types::urls::license_url::LicenseUrl;
use crate::types::urls::package_url::PackageUrl;
use crate::types::urls::publisher_support_url::PublisherSupportUrl;
use crate::types::urls::publisher_url::PublisherUrl;
use crate::types::urls::release_notes_url::ReleaseNotesUrl;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct DefaultLocaleManifest {
    pub package_identifier: PackageIdentifier,
    pub package_version: PackageVersion,
    pub package_locale: LanguageTag,
    pub publisher: Publisher,
    pub publisher_url: Option<PublisherUrl>,
    pub publisher_support_url: Option<PublisherSupportUrl>,
    pub privacy_url: Option<Url>,
    pub author: Option<Author>,
    pub package_name: PackageName,
    pub package_url: Option<PackageUrl>,
    pub license: License,
    pub license_url: Option<LicenseUrl>,
    pub copyright: Option<Copyright>,
    pub copyright_url: Option<CopyrightUrl>,
    pub short_description: ShortDescription,
    pub description: Option<Description>,
    pub moniker: Option<Moniker>,
    pub tags: Option<BTreeSet<Tag>>,
    pub agreements: Option<BTreeSet<Agreement>>,
    pub release_notes: Option<ReleaseNotes>,
    pub release_notes_url: Option<ReleaseNotesUrl>,
    pub purchase_url: Option<Url>,
    pub installation_notes: Option<InstallationNotes>,
    pub documentations: Option<BTreeSet<Documentation>>,
    pub icons: Option<BTreeSet<Icon>>,
    pub manifest_type: ManifestType,
    #[serde(default)]
    pub manifest_version: ManifestVersion,
}

impl ManifestTrait for DefaultLocaleManifest {
    const SCHEMA: &'static str = formatc!(
        "https://aka.ms/winget-manifest.defaultLocale.{}.schema.json",
        ManifestVersion::DEFAULT
    );
    const TYPE: ManifestType = ManifestType::DefaultLocale;
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Agreement {
    #[serde(rename = "AgreementLabel")]
    pub label: Option<String>,
    #[serde(rename = "Agreement")]
    pub text: Option<String>,
    #[serde(rename = "AgreementUrl")]
    pub url: Option<Url>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Documentation {
    pub document_label: Option<String>,
    pub document_url: Option<Url>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Icon {
    #[serde(rename = "IconUrl")]
    pub url: Url,
    #[serde(rename = "IconFileType")]
    pub file_type: Option<IconFileType>,
    #[serde(rename = "IconResolution")]
    pub resolution: Option<IconResolution>,
    #[serde(rename = "IconTheme")]
    pub theme: Option<IconTheme>,
    #[serde(rename = "IconSha256")]
    pub sha_256: Option<Sha256String>,
}

#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IconFileType {
    Png,
    Jpeg,
    Ico,
}

#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub enum IconResolution {
    Custom,
    #[serde(rename = "16x16")]
    Size16,
    #[serde(rename = "20x20")]
    Size20,
    #[serde(rename = "24x24")]
    Size24,
    #[serde(rename = "30x30")]
    Size30,
    #[serde(rename = "32x32")]
    Size32,
    #[serde(rename = "36x36")]
    Size36,
    #[serde(rename = "40x40")]
    Size40,
    #[serde(rename = "48x48")]
    Size48,
    #[serde(rename = "60x60")]
    Size60,
    #[serde(rename = "64x64")]
    Size64,
    #[serde(rename = "72x72")]
    Size72,
    #[serde(rename = "80x80")]
    Size80,
    #[serde(rename = "96x96")]
    Size96,
    #[serde(rename = "256x256")]
    Size256,
}

#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub enum IconTheme {
    Default,
    Light,
    Dark,
    HighContrast,
}

impl DefaultLocaleManifest {
    pub fn update(
        &mut self,
        package_version: &PackageVersion,
        github_values: &mut Option<GitHubValues>,
    ) {
        self.package_version.clone_from(package_version);
        if self.publisher_url.is_none() {
            self.publisher_url = github_values
                .as_mut()
                .map(|values| mem::take(&mut values.publisher_url));
        }
        if self.publisher_support_url.is_none() {
            self.publisher_support_url = github_values
                .as_mut()
                .and_then(|values| values.publisher_support_url.take());
        }
        if self.package_url.is_none() {
            self.package_url = github_values
                .as_ref()
                .map(|values| values.package_url.clone());
        }
        if let Some(github_license) = github_values
            .as_mut()
            .and_then(|values| values.license.take())
        {
            self.license = github_license;
        }
        if let Some(github_license_url) = github_values
            .as_mut()
            .and_then(|values| values.license_url.take())
        {
            self.license_url = Some(github_license_url);
        }
        if self.tags.is_none() {
            self.tags = github_values
                .as_mut()
                .and_then(|values| values.topics.take());
        }
        self.release_notes = github_values
            .as_mut()
            .and_then(|values| values.release_notes.take());
        self.release_notes_url = github_values
            .as_mut()
            .and_then(|values| values.release_notes_url.take());
        self.manifest_type = Self::TYPE;
        self.manifest_version = ManifestVersion::default();
    }
}
