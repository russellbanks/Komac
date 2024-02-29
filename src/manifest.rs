use crate::manifests::default_locale_manifest::DefaultLocaleManifest;
use crate::manifests::installer_manifest::InstallerManifest;
use crate::manifests::locale_manifest::LocaleManifest;
use crate::manifests::version_manifest::VersionManifest;
use clap::{crate_name, crate_version};
use color_eyre::eyre::{Error, Result};
use const_format::formatcp;
use crossterm::style::Stylize;
use std::io::StdoutLock;
use std::io::Write;
use std::{env, io};

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
    for line in manifest.lines() {
        if line.starts_with('#') {
            let _ = writeln!(lock, "{}", line.green());
        } else if let Some((prefix, suffix)) = line.split_once(':') {
            if let Some((before_dash, after_dash)) = prefix.split_once('-') {
                let _ = writeln!(lock, "{before_dash}-{}:{suffix}", after_dash.blue());
            } else {
                let _ = writeln!(lock, "{}:{suffix}", prefix.blue());
            }
        } else {
            let _ = writeln!(lock, "{line}");
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
