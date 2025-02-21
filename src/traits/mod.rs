use std::{collections::HashMap, mem, sync::LazyLock};

use derive_new::new;
use html2text::render::{TaggedLine, TextDecorator};
use regex::Regex;
use winget_types::{
    installer::Architecture,
    locale::{
        Copyright, DefaultLocaleManifest, LocaleManifest, PackageName, Publisher, ReleaseNotes,
    },
    shared::{ManifestVersion, PackageVersion, url::ReleaseNotesUrl},
    traits::Manifest,
};
use yara_x::mods::pe::Machine;

use crate::github::{github_client::GitHubValues, graphql::types::Html};

pub mod path;
pub mod url;

pub trait FromMachine {
    fn from_machine(machine: Machine) -> Self;
}

impl FromMachine for Architecture {
    fn from_machine(machine: Machine) -> Self {
        match machine {
            Machine::MACHINE_AMD64
            | Machine::MACHINE_IA64
            | Machine::MACHINE_POWERPC
            | Machine::MACHINE_POWERPCFP
            | Machine::MACHINE_R4000
            | Machine::MACHINE_SH5 => Self::X64,
            Machine::MACHINE_AM33
            | Machine::MACHINE_I386
            | Machine::MACHINE_M32R
            | Machine::MACHINE_SH3
            | Machine::MACHINE_SH3DSP
            | Machine::MACHINE_SH4 => Self::X86,
            Machine::MACHINE_ARM64 => Self::Arm64,
            Machine::MACHINE_ARM | Machine::MACHINE_ARMNT | Machine::MACHINE_THUMB => Self::Arm,
            Machine::MACHINE_UNKNOWN => Self::Neutral,
            machine => panic!("Unexpected architecture: {machine:?}"),
        }
    }
}

#[derive(new)]
struct GitHubHtmlDecorator;

impl TextDecorator for GitHubHtmlDecorator {
    type Annotation = ();

    fn decorate_link_start(&mut self, _url: &str) -> (String, Self::Annotation) {
        (String::new(), ())
    }

    fn decorate_link_end(&mut self) -> String {
        String::new()
    }

    fn decorate_em_start(&self) -> (String, Self::Annotation) {
        (String::new(), ())
    }

    fn decorate_em_end(&self) -> String {
        String::new()
    }

    fn decorate_strong_start(&self) -> (String, Self::Annotation) {
        (String::new(), ())
    }

    fn decorate_strong_end(&self) -> String {
        String::new()
    }

    fn decorate_strikeout_start(&self) -> (String, Self::Annotation) {
        (String::new(), ())
    }

    fn decorate_strikeout_end(&self) -> String {
        String::new()
    }

    fn decorate_code_start(&self) -> (String, Self::Annotation) {
        (String::new(), ())
    }

    fn decorate_code_end(&self) -> String {
        String::new()
    }

    fn decorate_preformat_first(&self) -> Self::Annotation {}

    fn decorate_preformat_cont(&self) -> Self::Annotation {}

    fn decorate_image(&mut self, _src: &str, title: &str) -> (String, Self::Annotation) {
        (title.to_string(), ())
    }

    fn header_prefix(&self, _level: usize) -> String {
        String::new()
    }

    fn quote_prefix(&self) -> String {
        String::from("> ")
    }

    fn unordered_item_prefix(&self) -> String {
        String::from("- ")
    }

    fn ordered_item_prefix(&self, i: i64) -> String {
        format!("{i}. ")
    }

    fn make_subblock_decorator(&self) -> Self {
        Self::new()
    }

    fn finalise(&mut self, _links: Vec<String>) -> Vec<TaggedLine<()>> {
        Vec::new()
    }
}

pub trait FromHtml {
    fn from_html(html: &Html) -> Option<Self>
    where
        Self: Sized;
}

impl FromHtml for ReleaseNotes {
    fn from_html(html: &Html) -> Option<Self> {
        // Strings that have whitespace before newlines get escaped and treated as literal strings
        // in yaml so this regex identifies any amount of whitespace and duplicate newlines
        static NEWLINE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+\n").unwrap());

        html2text::from_read_with_decorator(html.as_bytes(), usize::MAX, GitHubHtmlDecorator::new())
            .ok()
            .and_then(|text| Self::new(NEWLINE_REGEX.replace_all(&text, "\n")).ok())
    }
}

pub trait FromVSVersionInfo {
    fn from_version_info(version_info: &HashMap<String, String>) -> Option<Self>
    where
        Self: Sized;
}

impl FromVSVersionInfo for Copyright {
    fn from_version_info(version_info: &HashMap<String, String>) -> Option<Self> {
        version_info
            .get("LegalCopyright")
            .and_then(|product_name| Self::new(product_name.trim()).ok())
    }
}

impl FromVSVersionInfo for PackageName {
    fn from_version_info(version_info: &HashMap<String, String>) -> Option<Self> {
        version_info
            .get("ProductName")
            .and_then(|product_name| Self::new(product_name.trim()).ok())
    }
}

impl FromVSVersionInfo for Publisher {
    fn from_version_info(version_info: &HashMap<String, String>) -> Option<Self> {
        version_info
            .get("CompanyName")
            .and_then(|product_name| Self::new(product_name.trim()).ok())
    }
}

pub trait LocaleExt {
    fn update(
        &mut self,
        package_version: &PackageVersion,
        github_values: &mut Option<GitHubValues>,
        release_notes_url: Option<&ReleaseNotesUrl>,
    );
}

impl LocaleExt for LocaleManifest {
    fn update(
        &mut self,
        package_version: &PackageVersion,
        github_values: &mut Option<GitHubValues>,
        release_notes_url: Option<&ReleaseNotesUrl>,
    ) {
        self.package_version.clone_from(package_version);
        self.release_notes_url = release_notes_url.cloned().or_else(|| {
            github_values
                .as_ref()
                .and_then(|values| values.release_notes_url.clone())
        });
        self.manifest_type = Self::TYPE;
        self.manifest_version = ManifestVersion::default();
    }
}

impl LocaleExt for DefaultLocaleManifest {
    fn update(
        &mut self,
        package_version: &PackageVersion,
        github_values: &mut Option<GitHubValues>,
        release_notes_url: Option<&ReleaseNotesUrl>,
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
        self.release_notes_url = release_notes_url.cloned().or_else(|| {
            github_values
                .as_mut()
                .and_then(|values| values.release_notes_url.take())
        });
        self.manifest_type = Self::TYPE;
        self.manifest_version = ManifestVersion::default();
    }
}
