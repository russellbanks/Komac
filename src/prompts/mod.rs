use crate::manifests::installer_manifest::UpgradeBehavior;
use bitflags::Flags;
use inquire::error::InquireResult;
use inquire::{InquireError, MultiSelect, Select};
use std::fmt::Display;
use std::ops::BitOr;
use std::process;
use strum::IntoEnumIterator;
use winget::installer::command::Command;
use winget::installer::file_extension::FileExtension;
use winget::installer::install_modes::InstallModes;
use winget::installer::installer_return_code::InstallerReturnCode;
use winget::installer::protocol::Protocol;
use winget::installer::switches::custom::CustomSwitch;
use winget::installer::switches::silent::SilentSwitch;
use winget::installer::switches::silent_with_progress::SilentWithProgressSwitch;
use winget::locale::description::Description;
use winget::locale::license::License;
use winget::locale::moniker::Moniker;
use winget::locale::short_description::ShortDescription;
use winget::locale::tag::Tag;
use winget::shared::language_tag::LanguageTag;
use winget::shared::package_identifier::PackageIdentifier;
use winget::shared::package_version::PackageVersion;

pub mod list;
pub mod text;

pub trait Prompt {
    const MESSAGE: &'static str;
}

impl Prompt for PackageIdentifier {
    const MESSAGE: &'static str = "Package identifier:";
}

impl Prompt for PackageVersion {
    const MESSAGE: &'static str = "Package version:";
}

impl Prompt for UpgradeBehavior {
    const MESSAGE: &'static str = "Upgrade behaviour:";
}

impl Prompt for InstallModes {
    const MESSAGE: &'static str = "Install modes:";
}

impl Prompt for Protocol {
    const MESSAGE: &'static str = "Protocols:";
}

impl Prompt for FileExtension {
    const MESSAGE: &'static str = "File extensions:";
}

impl Prompt for Command {
    const MESSAGE: &'static str = "Commands:";
}

impl Prompt for InstallerReturnCode {
    const MESSAGE: &'static str = "Installer success codes:";
}

impl Prompt for Moniker {
    const MESSAGE: &'static str = "Moniker:";
}

impl Prompt for Tag {
    const MESSAGE: &'static str = "Tags:";
}

impl Prompt for Description {
    const MESSAGE: &'static str = "Description:";
}

impl Prompt for ShortDescription {
    const MESSAGE: &'static str = "Short description:";
}

impl Prompt for LanguageTag {
    const MESSAGE: &'static str = "Package locale:";
}

impl Prompt for License {
    const MESSAGE: &'static str = "License:";
}

impl Prompt for SilentSwitch {
    const MESSAGE: &'static str = "Silent installer switch:";
}

impl Prompt for SilentWithProgressSwitch {
    const MESSAGE: &'static str = "Silent with progress installer switch:";
}

impl Prompt for CustomSwitch {
    const MESSAGE: &'static str = "Custom installer switch:";
}

pub fn radio_prompt<T>() -> InquireResult<T>
where
    T: Prompt + IntoEnumIterator + Display,
{
    Select::new(T::MESSAGE, T::iter().collect())
        .prompt()
        .map_err(handle_inquire_error)
}

pub fn check_prompt<T>() -> InquireResult<Option<T>>
where
    T: Prompt + Flags + Display + BitOr<Output = T> + Copy,
{
    MultiSelect::new(T::MESSAGE, T::all().iter().collect())
        .prompt()
        .map(|items| {
            if items.is_empty() {
                None
            } else {
                Some(items.iter().fold(T::empty(), |flags, flag| flags | *flag))
            }
        })
        .map_err(handle_inquire_error)
}

/// Inquire captures Ctrl+C and returns an error. This will instead exit normally if the prompt is
/// interrupted.
pub fn handle_inquire_error(error: InquireError) -> InquireError {
    if matches!(
        error,
        InquireError::OperationCanceled | InquireError::OperationInterrupted
    ) {
        process::exit(0);
    }
    error
}
