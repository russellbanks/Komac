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
use crate::types::short_description::ShortDescription;
use crate::types::tag::Tag;
use crate::types::urls::copyright_url::CopyrightUrl;
use crate::types::urls::license_url::LicenseUrl;
use crate::types::urls::package_url::PackageUrl;
use crate::types::urls::publisher_url::PublisherUrl;
use crate::types::urls::release_notes_url::ReleaseNotesUrl;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::BTreeSet;
use url::Url;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct DefaultLocaleManifest {
    pub package_identifier: PackageIdentifier,
    pub package_version: PackageVersion,
    pub package_locale: LanguageTag,
    pub publisher: Publisher,
    pub publisher_url: Option<PublisherUrl>,
    pub publisher_support_url: Option<Url>,
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

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Agreement {
    pub agreement_label: Option<String>,
    pub agreement: Option<String>,
    pub agreement_url: Option<Url>,
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
    pub sha_256: Option<String>,
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
