use crate::default_locale_manifest::DefaultLocaleManifest;
use crate::installer_manifest::InstallerManifest;
use crate::locale_manifest::LocaleManifest;
use crate::version_manifest::VersionManifest;
use clap::{crate_name, crate_version};
use color_eyre::eyre::{Error, Result};
use const_format::formatcp;
use crossterm::style::Stylize;
use std::io::StdoutLock;
use std::io::Write;
use std::{env, io};

pub const MANIFEST_VERSION: &str = "1.5.0";

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

pub fn print_changes(changes: &Vec<(String, String)>) {
    let mut lock = io::stdout().lock();

    for (_, content) in changes {
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

pub fn build_manifest_string(manifest: Manifest) -> Result<String> {
    let mut result = Vec::from("# Created with ");
    if let Ok(created_with_tool) = env::var("KOMAC_CREATED_WITH") {
        write!(result, "{created_with_tool} using ")?;
    }
    writeln!(result, "{} v{}", crate_name!(), crate_version!())?;
    writeln!(
        result,
        "# yaml-language-server: $schema={}",
        match manifest {
            Manifest::Installer(_) => INSTALLER_SCHEMA,
            Manifest::DefaultLocale(_) => DEFAULT_LOCALE_SCHEMA,
            Manifest::Locale(_) => LOCALE_SCHEMA,
            Manifest::Version(_) => VERSION_SCHEMA,
        }
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
    use crate::installer_manifest::InstallerManifest;
    use crate::manifest::{build_manifest_string, Manifest};

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
        let manifest_string = build_manifest_string(installer_manifest).unwrap();
        assert!(!contains_newline_not_preceded_by_carriage_return(
            &manifest_string
        ));
    }
}
