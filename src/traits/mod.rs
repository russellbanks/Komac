mod ascii_ext;
pub mod name;
pub mod path;
use std::{collections::HashMap, mem, sync::LazyLock};

pub use ascii_ext::AsciiExt;
use html2text::render::{TaggedLine, TextDecorator};
pub use name::Name;
use regex::Regex;
use winget_types::{
    Manifest, ManifestVersion, PackageVersion,
    installer::Architecture,
    locale::{
        Copyright, DefaultLocaleManifest, LocaleManifest, PackageName, Publisher, ReleaseNotes,
    },
    url::ReleaseNotesUrl,
};

use super::{
    analysis::installers::pe::{
        IMAGE_FILE_MACHINE_AM33, IMAGE_FILE_MACHINE_AMD64, IMAGE_FILE_MACHINE_ARM,
        IMAGE_FILE_MACHINE_ARM64, IMAGE_FILE_MACHINE_ARM64EC, IMAGE_FILE_MACHINE_ARM64X,
        IMAGE_FILE_MACHINE_ARMNT, IMAGE_FILE_MACHINE_I386, IMAGE_FILE_MACHINE_IA64,
        IMAGE_FILE_MACHINE_M32R, IMAGE_FILE_MACHINE_POWERPC, IMAGE_FILE_MACHINE_POWERPCFP,
        IMAGE_FILE_MACHINE_R4000, IMAGE_FILE_MACHINE_SH3, IMAGE_FILE_MACHINE_SH3DSP,
        IMAGE_FILE_MACHINE_SH4, IMAGE_FILE_MACHINE_SH5, IMAGE_FILE_MACHINE_THUMB,
        IMAGE_FILE_MACHINE_UNKNOWN,
    },
    github::{client::GitHubValues, graphql::types::Html},
};
use crate::analysis::installers::pe::PE;

pub trait FromMachine {
    fn from_machine(machine: u16) -> Self;
}

impl FromMachine for Architecture {
    fn from_machine(machine: u16) -> Self {
        match machine {
            IMAGE_FILE_MACHINE_AMD64
            | IMAGE_FILE_MACHINE_IA64
            | IMAGE_FILE_MACHINE_POWERPC
            | IMAGE_FILE_MACHINE_POWERPCFP
            | IMAGE_FILE_MACHINE_R4000
            | IMAGE_FILE_MACHINE_SH5 => Self::X64,
            IMAGE_FILE_MACHINE_AM33
            | IMAGE_FILE_MACHINE_I386
            | IMAGE_FILE_MACHINE_M32R
            | IMAGE_FILE_MACHINE_SH3
            | IMAGE_FILE_MACHINE_SH3DSP
            | IMAGE_FILE_MACHINE_SH4 => Self::X86,
            IMAGE_FILE_MACHINE_ARM64 | IMAGE_FILE_MACHINE_ARM64EC | IMAGE_FILE_MACHINE_ARM64X => {
                Self::Arm64
            }
            IMAGE_FILE_MACHINE_ARM | IMAGE_FILE_MACHINE_ARMNT | IMAGE_FILE_MACHINE_THUMB => {
                Self::Arm
            }
            IMAGE_FILE_MACHINE_UNKNOWN => Self::Neutral,
            _ => panic!("Unexpected architecture: {machine:?}"),
        }
    }
}

pub trait IntoWingetArchitecture {
    fn winget_architecture(&self) -> Architecture;
}

impl IntoWingetArchitecture for PE {
    fn winget_architecture(&self) -> Architecture {
        Architecture::from_machine(self.machine())
    }
}

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
        Self
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

        html2text::from_read_with_decorator(html.as_bytes(), usize::MAX, GitHubHtmlDecorator)
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
            github_values.as_ref().and_then(|values| {
                if values.release_notes.is_some() {
                    values.release_notes_url.clone()
                } else {
                    None
                }
            })
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
                .and_then(|values| values.issues_url.take());
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
        if self.tags.is_empty() {
            self.tags = github_values
                .as_mut()
                .map(|values| mem::take(&mut values.topics))
                .unwrap_or_default();
        }
        self.release_notes = github_values
            .as_mut()
            .and_then(|values| values.release_notes.take());
        self.release_notes_url = release_notes_url.cloned().or_else(|| {
            github_values.as_mut().and_then(|values| {
                if self.release_notes.is_some() {
                    values.release_notes_url.take()
                } else {
                    None
                }
            })
        });
        self.manifest_type = Self::TYPE;
        self.manifest_version = ManifestVersion::default();
    }
}
