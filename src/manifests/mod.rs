use std::{
    borrow::Cow,
    env,
    fmt::{Display, Formatter, Write},
    io::{StdoutLock, Write as IoWrite},
    sync::LazyLock,
};

use anstream::AutoStream;
use clap::{crate_name, crate_version};
use const_format::concatc;
use owo_colors::{OwoColorize, Style, colors::css::SlateGrey};
use serde::Serialize;
use tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent, Highlighter};
use winget_types::{
    installer::InstallerManifest,
    locale::{DefaultLocaleManifest, LocaleManifest},
    traits::Manifest,
    version::VersionManifest,
};

pub mod manifest;

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

pub fn build_manifest_string<T>(
    manifest: &T,
    created_with: Option<&str>,
) -> serde_yaml::Result<String>
where
    T: Manifest + Serialize,
{
    let mut result = String::from("# Created with ");
    if let Some(created_with_tool) = created_with {
        let _ = write!(result, "{created_with_tool} using ");
    }
    let _ = writeln!(result, "{} v{}", crate_name!(), crate_version!());
    let _ = writeln!(result, "# yaml-language-server: $schema={}", T::SCHEMA);
    let _ = writeln!(result);
    let _ = writeln!(result, "{}", serde_yaml::to_string(manifest)?);
    Ok(convert_to_crlf(&result).into_owned())
}

fn convert_to_crlf(input: &str) -> Cow<str> {
    const CR: char = '\r';
    const LF: char = '\n';
    const CRLF: &str = concatc!(CR, LF);

    let mut buffer = None;
    let mut position = 0;
    let mut chars = input.char_indices().peekable();

    while let Some((index, char)) = chars.next() {
        match char {
            CR => {
                let buf = buffer.get_or_insert_with(|| String::with_capacity(input.len()));

                // Copy text before CR
                buf.push_str(&input[position..index]);

                // Check for CR+LF
                if let Some(&(_, LF)) = chars.peek() {
                    // Skip the LF as we'll add CRLF
                    chars.next();
                }

                buf.push_str(CRLF);

                position = chars
                    .peek()
                    .map_or(input.len(), |&(next_index, _)| next_index);
            }
            LF => {
                // Convert LF
                let buf = buffer.get_or_insert_with(|| String::with_capacity(input.len()));
                buf.push_str(&input[position..index]);
                buf.push_str(CRLF);
                position = index + LF.len_utf8();
            }
            _ => {}
        }
    }

    buffer.map_or(Cow::Borrowed(input), |mut buf| {
        buf.push_str(&input[position..]);
        Cow::Owned(buf)
    })
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::manifests::convert_to_crlf;

    #[test]
    fn preserves_valid_crlf() {
        assert_eq!(
            convert_to_crlf("Valid\r\nLine"),
            Cow::Borrowed("Valid\r\nLine")
        );
    }

    #[test]
    fn converts_lf_to_crlf() {
        assert_eq!(
            convert_to_crlf("Unix\nLine"),
            Cow::Owned::<str>("Unix\r\nLine".into())
        );
    }

    #[test]
    fn converts_lone_cr_to_crlf() {
        assert_eq!(
            convert_to_crlf("Old\rMac"),
            Cow::Owned::<str>("Old\r\nMac".into())
        );
    }

    #[test]
    fn mixed_conversions() {
        assert_eq!(
            convert_to_crlf("Mix\r\n\n\rEnd"),
            Cow::Owned::<str>("Mix\r\n\r\n\r\nEnd".into())
        );
    }

    #[test]
    fn no_changes_needed() {
        assert_eq!(convert_to_crlf("No changes"), Cow::Borrowed("No changes"));
    }

    #[test]
    fn empty_string() {
        assert_eq!(convert_to_crlf(""), Cow::Borrowed(""));
    }

    #[test]
    fn edge_cases() {
        assert_eq!(convert_to_crlf("\r"), "\r\n");
        assert_eq!(convert_to_crlf("\n"), "\r\n");
        assert_eq!(convert_to_crlf("\r\n"), "\r\n");
        assert_eq!(convert_to_crlf("a\rb\nc\r\nd"), "a\r\nb\r\nc\r\nd");
    }
}
