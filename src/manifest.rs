use std::io::StdoutLock;
use std::io::Write;
use std::{env, io};

use clap::{crate_name, crate_version};
use color_eyre::eyre::{Error, Result};
use const_format::formatcp;
use crossterm::style::{style, Color, Stylize};
use tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent, Highlighter};

use crate::manifests::default_locale_manifest::DefaultLocaleManifest;
use crate::manifests::installer_manifest::InstallerManifest;
use crate::manifests::locale_manifest::LocaleManifest;
use crate::manifests::version_manifest::VersionManifest;

pub const MANIFEST_VERSION: &str = "1.6.0";

const INSTALLER_SCHEMA: &str =
    formatcp!("https://aka.ms/winget-manifest.installer.{MANIFEST_VERSION}.schema.json");
const DEFAULT_LOCALE_SCHEMA: &str =
    formatcp!("https://aka.ms/winget-manifest.defaultLocale.{MANIFEST_VERSION}.schema.json");
const LOCALE_SCHEMA: &str =
    formatcp!("https://aka.ms/winget-manifest.locale.{MANIFEST_VERSION}.schema.json");
const VERSION_SCHEMA: &str =
    formatcp!("https://aka.ms/winget-manifest.version.{MANIFEST_VERSION}.schema.json");

pub enum Manifest<'a> {
    Installer(&'a InstallerManifest),
    DefaultLocale(&'a DefaultLocaleManifest),
    Locale(&'a LocaleManifest),
    Version(&'a VersionManifest),
}

impl Manifest<'_> {
    const fn schema(&self) -> &str {
        match self {
            Manifest::Installer(_) => INSTALLER_SCHEMA,
            Manifest::DefaultLocale(_) => DEFAULT_LOCALE_SCHEMA,
            Manifest::Locale(_) => LOCALE_SCHEMA,
            Manifest::Version(_) => VERSION_SCHEMA,
        }
    }
}

pub fn print_changes<'a>(contents: impl Iterator<Item = &'a str>) {
    let mut lock = io::stdout().lock();

    for content in contents {
        print_manifest(&mut lock, content);
        let _ = writeln!(lock);
    }
}

fn print_manifest(lock: &mut StdoutLock, manifest: &str) {
    const COMMENT: &str = "comment";
    const PROPERTY: &str = "property";
    const STRING: &str = "string";
    const HIGHLIGHT_NAMES: [&str; 3] = [COMMENT, STRING, PROPERTY];

    let mut highlighter = Highlighter::new();

    let mut yaml_config = HighlightConfiguration::new(
        tree_sitter_yaml::language(),
        "yaml",
        tree_sitter_yaml::HIGHLIGHTS_QUERY,
        <&str>::default(),
        <&str>::default(),
    )
    .unwrap();
    yaml_config.configure(&HIGHLIGHT_NAMES);
    let highlights = highlighter
        .highlight(&yaml_config, manifest.as_bytes(), None, |_| None)
        .unwrap();

    let mut current_highlight = None;
    for event in highlights {
        match event.unwrap() {
            HighlightEvent::Source { start, end } => {
                let source = &manifest[start..end];
                let _ = write!(
                    lock,
                    "{}",
                    style(source).with(
                        current_highlight
                            .and_then(|value: Highlight| {
                                match HIGHLIGHT_NAMES[value.0] {
                                    COMMENT => Some(Color::DarkGrey),
                                    PROPERTY => Some(Color::Green),
                                    STRING => {
                                        if source.chars().all(|char| {
                                            char.is_ascii_digit() || char.is_ascii_punctuation()
                                        }) {
                                            Some(Color::Blue)
                                        } else {
                                            None
                                        }
                                    }
                                    _ => None,
                                }
                            })
                            .unwrap_or(Color::Reset)
                    )
                );
            }
            HighlightEvent::HighlightStart(highlight) => current_highlight = Some(highlight),
            HighlightEvent::HighlightEnd => current_highlight = None,
        }
    }
}

pub fn build_manifest_string(manifest: &Manifest, created_with: &Option<String>) -> Result<String> {
    let mut result = Vec::from("# Created with ");
    if let Some(created_with_tool) = created_with {
        write!(result, "{created_with_tool} using ")?;
    }
    writeln!(result, "{} v{}", crate_name!(), crate_version!())?;
    writeln!(
        result,
        "# yaml-language-server: $schema={}",
        manifest.schema()
    )?;
    writeln!(result)?;
    match manifest {
        Manifest::Installer(manifest) => serde_yaml::to_writer(&mut result, manifest)?,
        Manifest::DefaultLocale(manifest) => serde_yaml::to_writer(&mut result, manifest)?,
        Manifest::Locale(manifest) => serde_yaml::to_writer(&mut result, manifest)?,
        Manifest::Version(manifest) => serde_yaml::to_writer(&mut result, manifest)?,
    }
    convert_to_crlf(&mut result);
    String::from_utf8(result).map_err(Error::msg)
}

fn convert_to_crlf(buf: &mut Vec<u8>) {
    const NEWLINE: u8 = b'\n';
    const CARRIAGE_RETURN: u8 = b'\r';

    let mut prev_char: Option<u8> = None;
    let mut i = 0;
    while i < buf.len() {
        // Check whether the character is a newline and is not preceded by a carriage return
        if buf[i] == NEWLINE && prev_char != Some(CARRIAGE_RETURN) {
            // Insert a carriage return before the newline
            buf.insert(i, CARRIAGE_RETURN);
            i += 1; // Move to the next character to avoid infinite loop
        }
        prev_char = Some(buf[i]);
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::manifest::{build_manifest_string, Manifest};
    use crate::manifests::installer_manifest::InstallerManifest;

    fn contains_newline_not_preceded_by_carriage_return(value: &str) -> bool {
        value
            .chars()
            .zip(value.chars().skip(1))
            .any(|(prev, current)| prev != '\r' && current == '\n')
    }

    #[test]
    fn test_build_manifest_string_crlf() {
        let binding = InstallerManifest::default();
        let installer_manifest = Manifest::Installer(&binding);
        let manifest_string = build_manifest_string(&installer_manifest, &None).unwrap();
        assert!(!contains_newline_not_preceded_by_carriage_return(
            &manifest_string
        ));
    }
}
