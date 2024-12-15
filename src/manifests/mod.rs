use crate::manifests::default_locale_manifest::DefaultLocaleManifest;
use crate::manifests::installer_manifest::InstallerManifest;
use crate::manifests::locale_manifest::LocaleManifest;
use crate::manifests::version_manifest::VersionManifest;
use crate::types::manifest_type::ManifestType;
use anstream::AutoStream;
use clap::{crate_name, crate_version};
use color_eyre::eyre::{Error, Result};
use owo_colors::colors::css::SlateGrey;
use owo_colors::{OwoColorize, Style};
use serde::Serialize;
use std::env;
use std::fmt::{Display, Formatter};
use std::io::StdoutLock;
use std::io::Write;
use std::sync::LazyLock;
use tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent, Highlighter};

pub mod default_locale_manifest;
pub mod installer_manifest;
pub mod locale_manifest;
pub mod version_manifest;

pub trait Manifest {
    const SCHEMA: &'static str;

    const TYPE: ManifestType;
}

pub struct Manifests {
    pub installer: InstallerManifest,
    pub default_locale: DefaultLocaleManifest,
    pub locales: Vec<LocaleManifest>,
    pub version: VersionManifest,
}

impl Display for Manifests {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} version {}",
            self.version.package_identifier, self.version.package_version
        )
    }
}

pub fn print_changes<'a>(contents: impl Iterator<Item = &'a str>) {
    let mut lock = anstream::stdout().lock();

    for content in contents {
        print_manifest(&mut lock, content);
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
                        STRING => {
                            if source
                                .chars()
                                .all(|char| char.is_ascii_digit() || char.is_ascii_punctuation())
                            {
                                style = style.blue();
                            }
                        }
                        _ => {}
                    }
                }
                let _ = write!(lock, "{}", source.style(style));
            }
            Ok(HighlightEvent::HighlightStart(highlight)) => current_highlight = Some(highlight),
            Ok(HighlightEvent::HighlightEnd) => current_highlight = None,
            Err(_) => continue,
        }
    }
}

pub fn build_manifest_string<T: Manifest + Serialize>(
    manifest: &T,
    created_with: Option<&str>,
) -> Result<String> {
    let mut result = Vec::from("# Created with ");
    if let Some(created_with_tool) = created_with {
        write!(result, "{created_with_tool} using ")?;
    }
    writeln!(result, "{} v{}", crate_name!(), crate_version!())?;
    writeln!(result, "# yaml-language-server: $schema={}", T::SCHEMA)?;
    writeln!(result)?;
    serde_yaml::to_writer(&mut result, manifest)?;
    convert_to_crlf(&mut result);
    String::from_utf8(result).map_err(Error::from)
}

fn convert_to_crlf(buf: &mut Vec<u8>) {
    const NEWLINE: u8 = b'\n';
    const CARRIAGE_RETURN: u8 = b'\r';

    let mut prev_char: Option<u8> = None;
    let mut index = 0;
    while index < buf.len() {
        // Check whether the character is a newline and is not preceded by a carriage return
        if buf[index] == NEWLINE && prev_char != Some(CARRIAGE_RETURN) {
            // Insert a carriage return before the newline
            buf.insert(index, CARRIAGE_RETURN);
            index += 1; // Move to the next character to avoid infinite loop
        }
        prev_char = Some(buf[index]);
        index += 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::manifests::convert_to_crlf;
    use std::io::Write;

    fn is_line_feed(value: &str) -> bool {
        value
            .chars()
            .zip(value.chars().skip(1))
            .any(|(prev, current)| prev != '\r' && current == '\n')
    }

    #[test]
    fn test_convert_to_crlf() {
        let mut buffer = Vec::new();
        for index in 0..10 {
            let _ = writeln!(buffer, "Line {index}");
        }
        assert!(is_line_feed(std::str::from_utf8(&buffer).unwrap()));
        convert_to_crlf(&mut buffer);
        assert!(!is_line_feed(std::str::from_utf8(&buffer).unwrap()));
    }
}
