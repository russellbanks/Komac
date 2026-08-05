use std::{
    fmt,
    fmt::Display,
    io::{StdoutLock, Write as IoWrite},
    sync::LazyLock,
};

use anstream::AutoStream;
use owo_colors::{OwoColorize, Style, colors::css::SlateGrey};
use tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent, Highlighter};
pub use url::Url;
use winget_types::{
    Manifest, PackageIdentifier, PackageVersion, VersionManifest,
    installer::InstallerManifest,
    locale::{DefaultLocaleManifest, LocaleManifest},
    url::ReleaseNotesUrl,
};

use crate::{
    github::{
        client::GitHubValues,
        utils::{
            PackagePath,
            pull_request::{Change, Changes},
        },
    },
    traits::LocaleExt,
};

pub mod manifest;
mod url;

pub struct Manifests {
    pub installer: InstallerManifest,
    pub default_locale: DefaultLocaleManifest,
    pub locales: Vec<LocaleManifest>,
    pub version: VersionManifest,
}

impl Manifests {
    /// Returns the package identifier for this package, retrieved from the
    /// version manifest.
    #[must_use]
    #[inline]
    pub fn package_identifier(&self) -> &PackageIdentifier {
        self.version.package_identifier()
    }

    /// Returns the package version for this package, retrieved from the
    /// version manifest.
    #[must_use]
    #[inline]
    pub fn package_version(&self) -> &PackageVersion {
        self.version.package_version()
    }

    pub fn create(
        &self,
        identifier: &PackageIdentifier,
        version: &PackageVersion,
        created_with: Option<&str>,
    ) -> Changes {
        let package_path = PackagePath::new(identifier, Some(version), None);

        let mut path_content_map = vec![
            Change::new(
                format!("{package_path}/{identifier}.installer.yaml"),
                &self.installer,
                created_with,
            ),
            Change::new(
                format!(
                    "{package_path}/{identifier}.locale.{}.yaml",
                    self.version.default_locale()
                ),
                &self.default_locale,
                created_with,
            ),
        ];

        for locale_manifest in &self.locales {
            path_content_map.push(Change::new(
                format!(
                    "{package_path}/{identifier}.locale.{}.yaml",
                    locale_manifest.package_locale
                ),
                locale_manifest,
                created_with,
            ));
        }

        path_content_map.push(Change::new(
            format!("{package_path}/{identifier}.yaml"),
            &self.version,
            created_with,
        ));

        Changes::new(path_content_map)
    }

    pub fn update(
        &mut self,
        version: &PackageVersion,
        github_values: &mut Option<GitHubValues>,
        release_notes_url: Option<&ReleaseNotesUrl>,
    ) {
        self.default_locale
            .update(version, github_values, release_notes_url);

        self.locales.iter_mut().for_each(|locale| {
            locale.update(version, github_values, release_notes_url);
        });

        self.version.update(version);
    }
}

impl Display for Manifests {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} version {}",
            self.package_identifier(),
            self.package_version()
        )
    }
}

pub fn print_changes<I, S>(contents: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut lock = anstream::stdout().lock();

    for content in contents {
        print_manifest(&mut lock, content.as_ref());
        let _ = writeln!(lock);
    }
}

pub fn print_manifest(lock: &mut AutoStream<StdoutLock<'static>>, manifest: &str) {
    const COMMENT: &str = "comment";
    const PROPERTY: &str = "property";
    const STRING: &str = "string";
    const HIGHLIGHT_NAMES: [&str; 3] = [COMMENT, STRING, PROPERTY];
    const YAML: &str = "yaml";

    static YAML_CONFIG: LazyLock<HighlightConfiguration> = LazyLock::new(|| {
        let mut config = HighlightConfiguration::new(
            tree_sitter_yaml::LANGUAGE.into(),
            YAML,
            tree_sitter_yaml::HIGHLIGHTS_QUERY,
            <&str>::default(),
            <&str>::default(),
        )
        .unwrap();
        config.configure(&HIGHLIGHT_NAMES);
        config
    });

    let mut highlighter = Highlighter::new();
    let highlights = highlighter
        .highlight(&YAML_CONFIG, manifest.as_bytes(), None, |_| None)
        .unwrap();

    let mut current_highlight: Option<Highlight> = None;
    for event in highlights {
        match event {
            Ok(HighlightEvent::Source { start, end }) => {
                let source = &manifest[start..end];
                let mut style = Style::new();
                if let Some(highlight) = current_highlight {
                    match HIGHLIGHT_NAMES[highlight.0] {
                        COMMENT => style = style.fg::<SlateGrey>(),
                        PROPERTY => style = style.green(),
                        STRING
                            if source.chars().all(|char| {
                                char.is_ascii_digit() || char.is_ascii_punctuation()
                            }) =>
                        {
                            style = style.blue();
                        }
                        _ => {}
                    }
                }
                let _ = write!(lock, "{}", source.style(style));
            }
            Ok(HighlightEvent::HighlightStart(highlight)) => current_highlight = Some(highlight),
            Ok(HighlightEvent::HighlightEnd) => current_highlight = None,
            Err(_) => {}
        }
    }
}
