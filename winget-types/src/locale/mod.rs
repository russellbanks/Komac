mod agreement;
mod author;
mod copyright;
mod description;
mod icon;
mod installation_notes;
mod license;
mod moniker;
mod package_name;
mod publisher;
mod release_notes;
mod short_description;
mod tag;

use std::collections::BTreeSet;

use const_format::formatc;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

pub use crate::locale::{
    agreement::Agreement, author::Author, copyright::Copyright, description::Description,
    icon::Icon, installation_notes::InstallationNotes, license::License, moniker::Moniker,
    package_name::PackageName, publisher::Publisher, release_notes::ReleaseNotes,
    short_description::ShortDescription, tag::Tag,
};
use crate::{
    shared::{
        LanguageTag, ManifestType, ManifestVersion, PackageIdentifier, PackageVersion,
        url::{
            CopyrightUrl, LicenseUrl, PackageUrl, PublisherSupportUrl, PublisherUrl,
            ReleaseNotesUrl,
        },
    },
    traits::Manifest,
};

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

impl Manifest for DefaultLocaleManifest {
    const SCHEMA: &'static str = formatc!(
        "https://aka.ms/winget-manifest.defaultLocale.{}.schema.json",
        ManifestVersion::DEFAULT
    );
    const TYPE: ManifestType = ManifestType::DefaultLocale;
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Documentation {
    pub document_label: Option<String>,
    pub document_url: Option<Url>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LocaleManifest {
    pub package_identifier: PackageIdentifier,
    pub package_version: PackageVersion,
    pub package_locale: LanguageTag,
    pub publisher: Option<Publisher>,
    pub publisher_url: Option<Url>,
    pub publisher_support_url: Option<Url>,
    pub privacy_url: Option<Url>,
    pub author: Option<Author>,
    pub package_name: Option<PackageName>,
    pub package_url: Option<Url>,
    pub license: Option<License>,
    pub license_url: Option<Url>,
    pub copyright: Option<Copyright>,
    pub copyright_url: Option<Url>,
    pub short_description: Option<ShortDescription>,
    pub description: Option<Description>,
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

impl Manifest for LocaleManifest {
    const SCHEMA: &'static str = formatc!(
        "https://aka.ms/winget-manifest.locale.{}.schema.json",
        ManifestVersion::DEFAULT
    );
    const TYPE: ManifestType = ManifestType::Locale;
}
