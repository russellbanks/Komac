use crate::manifests::default_locale_manifest::{Agreement, Documentation, Icon};
use crate::types::author::Author;
use crate::types::copyright::Copyright;
use crate::types::description::Description;
use crate::types::installation_notes::InstallationNotes;
use crate::types::language_tag::LanguageTag;
use crate::types::license::License;
use crate::types::manifest_type::ManifestType;
use crate::types::manifest_version::ManifestVersion;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_name::PackageName;
use crate::types::package_version::PackageVersion;
use crate::types::publisher::Publisher;
use crate::types::release_notes::ReleaseNotes;
use crate::types::short_description::ShortDescription;
use crate::types::tag::Tag;
use crate::types::urls::release_notes_url::ReleaseNotesUrl;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::BTreeSet;
use url::Url;

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
