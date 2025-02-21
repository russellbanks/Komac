use crate::prompts::{Prompt, handle_inquire_error};
use inquire::error::InquireResult;
use inquire::validator::Validation;
use inquire::{Confirm, CustomUserError, InquireError, Text};
use std::fmt::{Debug, Display};
use std::str::FromStr;
use winget::installer::switches::custom::CustomSwitch;
use winget::installer::switches::silent::SilentSwitch;
use winget::installer::switches::silent_with_progress::SilentWithProgressSwitch;
use winget::locale::description::Description;
use winget::locale::license::License;
use winget::locale::moniker::Moniker;
use winget::locale::short_description::ShortDescription;
use winget::shared::language_tag::LanguageTag;
use winget::shared::package_identifier::PackageIdentifier;
use winget::shared::package_version::PackageVersion;

pub trait TextPrompt: Prompt {
    const HELP_MESSAGE: Option<&'static str> = None;
    const PLACEHOLDER: Option<&'static str> = None;
}

impl TextPrompt for PackageIdentifier {
    const HELP_MESSAGE: Option<&'static str> =
        Some("Package Identifiers are in the format of Package.Identifier");
    const PLACEHOLDER: Option<&'static str> = Some("Package.Identifier");
}

impl TextPrompt for PackageVersion {
    const HELP_MESSAGE: Option<&'static str> = Some("Example: 1.2.3");
}

impl TextPrompt for Moniker {
    const HELP_MESSAGE: Option<&'static str> = Some("Example: vscode");
}

impl TextPrompt for Description {}

impl TextPrompt for ShortDescription {}

impl TextPrompt for LanguageTag {
    const HELP_MESSAGE: Option<&'static str> = Some("Example: en-US");
}

impl TextPrompt for License {
    const HELP_MESSAGE: Option<&'static str> = Some("Example: MIT, GPL-3.0, Freeware, Proprietary");
}

impl TextPrompt for SilentSwitch {
    const HELP_MESSAGE: Option<&'static str> =
        Some("Example: /S, -verysilent, /qn, --silent, /exenoui");
    const PLACEHOLDER: Option<&'static str> = None;
}

impl TextPrompt for SilentWithProgressSwitch {
    const HELP_MESSAGE: Option<&'static str> = Some("Example: /S, -silent, /qb, /exebasicui");
}

impl TextPrompt for CustomSwitch {
    const HELP_MESSAGE: Option<&'static str> = Some("Example: /norestart, -norestart");
    const PLACEHOLDER: Option<&'static str> = None;
}

pub fn optional_prompt<T>(parameter: Option<T>) -> InquireResult<Option<T>>
where
    T: FromStr + TextPrompt,
    <T as FromStr>::Err: Display + Debug + Sync + Send + 'static,
{
    if let Some(value) = parameter {
        Ok(Some(value))
    } else {
        let mut prompt = Text::new(T::MESSAGE).with_validator(|input: &str| {
            if input.is_empty() {
                Ok(Validation::Valid)
            } else {
                match input.parse::<T>() {
                    Ok(_) => Ok(Validation::Valid),
                    Err(error) => Ok(Validation::Invalid(error.into())),
                }
            }
        });
        if let Some(help_message) = T::HELP_MESSAGE {
            prompt = prompt.with_help_message(help_message);
        }
        if let Some(placeholder) = T::PLACEHOLDER {
            prompt = prompt.with_placeholder(placeholder);
        }
        let result = prompt.prompt().map_err(handle_inquire_error)?;
        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result.parse::<T>().map_err(|err| {
                InquireError::from(CustomUserError::from(err.to_string()))
            })?))
        }
    }
}

pub fn required_prompt<T>(parameter: Option<T>) -> InquireResult<T>
where
    T: FromStr + TextPrompt,
    <T as FromStr>::Err: ToString,
{
    if let Some(value) = parameter {
        Ok(value)
    } else {
        let mut prompt =
            Text::new(T::MESSAGE).with_validator(|input: &str| match input.parse::<T>() {
                Ok(_) => Ok(Validation::Valid),
                Err(error) => Ok(Validation::Invalid(error.into())),
            });
        if let Some(help_message) = T::HELP_MESSAGE {
            prompt = prompt.with_help_message(help_message);
        }
        if let Some(placeholder) = T::PLACEHOLDER {
            prompt = prompt.with_placeholder(placeholder);
        }
        prompt
            .prompt()
            .map_err(handle_inquire_error)?
            .parse::<T>()
            .map_err(|err| InquireError::from(CustomUserError::from(err.to_string())))
    }
}

pub fn confirm_prompt(message: &str) -> InquireResult<bool> {
    Confirm::new(message)
        .with_placeholder("y/n")
        .prompt()
        .map_err(handle_inquire_error)
}
