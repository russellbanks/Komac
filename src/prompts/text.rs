use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use inquire::{
    Confirm, CustomUserError, InquireError, Text, error::InquireResult, validator::Validation,
};
use winget_types::{
    LanguageTag, PackageIdentifier, PackageVersion,
    installer::{
        PortableCommandAlias,
        switches::{CustomSwitch, SilentSwitch, SilentWithProgressSwitch},
    },
    locale::{
        Author, Copyright, Description, License, Moniker, PackageName, Publisher, ShortDescription,
    },
    url::{
        CopyrightUrl, LicenseUrl, PackageUrl, PublisherSupportUrl, PublisherUrl, ReleaseNotesUrl,
    },
};

use crate::{i18n::fl, prompts::handle_inquire_error, traits::Name};

pub trait TextPrompt: Name {
    /// Returns the translated help message shown below the prompt
    fn help_message() -> Option<String> {
        None
    }

    /// Returns the translated placeholder shown in the prompt while it's empty
    fn placeholder() -> Option<String> {
        None
    }
}

impl TextPrompt for PackageIdentifier {
    fn help_message() -> Option<String> {
        Some(fl!("field-package-identifier", "help"))
    }

    fn placeholder() -> Option<String> {
        Some(fl!("field-package-identifier", "placeholder"))
    }
}

impl TextPrompt for PackageVersion {
    fn help_message() -> Option<String> {
        Some(fl!("field-package-version", "help"))
    }
}

impl TextPrompt for Moniker {
    fn help_message() -> Option<String> {
        Some(fl!("field-moniker", "help"))
    }
}

impl TextPrompt for Description {}

impl TextPrompt for ShortDescription {}

impl TextPrompt for LanguageTag {
    fn help_message() -> Option<String> {
        Some(fl!("field-language-tag", "help"))
    }
}

impl TextPrompt for Publisher {
    fn help_message() -> Option<String> {
        Some(fl!("field-publisher", "help"))
    }
}

impl TextPrompt for License {
    fn help_message() -> Option<String> {
        Some(fl!("field-license", "help"))
    }
}

impl TextPrompt for SilentSwitch {
    fn help_message() -> Option<String> {
        Some(fl!("field-silent-switch", "help"))
    }
}

impl TextPrompt for SilentWithProgressSwitch {
    fn help_message() -> Option<String> {
        Some(fl!("field-silent-with-progress-switch", "help"))
    }
}

impl TextPrompt for CustomSwitch {
    fn help_message() -> Option<String> {
        Some(fl!("field-custom-switch", "help"))
    }
}

impl TextPrompt for Copyright {
    fn help_message() -> Option<String> {
        Some(fl!("field-copyright", "help"))
    }
}

impl TextPrompt for Author {}

impl TextPrompt for PublisherUrl {}

impl TextPrompt for PackageName {
    fn help_message() -> Option<String> {
        Some(fl!("field-package-name", "help"))
    }
}

impl TextPrompt for PackageUrl {}

impl TextPrompt for LicenseUrl {}

impl TextPrompt for PublisherSupportUrl {}

impl TextPrompt for CopyrightUrl {}

impl TextPrompt for ReleaseNotesUrl {}

impl TextPrompt for PortableCommandAlias {}

pub fn optional_prompt<T, U>(parameter: Option<T>, default: Option<U>) -> InquireResult<Option<T>>
where
    T: FromStr + TextPrompt,
    <T as FromStr>::Err: Display + Debug + Sync + Send + 'static,
    U: AsRef<str>,
{
    if let Some(value) = parameter {
        Ok(Some(value))
    } else {
        let message = format!("{}:", T::name());
        let mut prompt = Text::new(&message).with_validator(|input: &str| {
            if input.is_empty() {
                Ok(Validation::Valid)
            } else {
                match input.parse::<T>() {
                    Ok(_) => Ok(Validation::Valid),
                    Err(error) => Ok(Validation::Invalid(error.into())),
                }
            }
        });
        let help_message = T::help_message();
        if let Some(help_message) = &help_message {
            prompt = prompt.with_help_message(help_message);
        }
        let placeholder = T::placeholder();
        if let Some(placeholder) = &placeholder {
            prompt = prompt.with_placeholder(placeholder);
        }
        let default = default.as_ref().map(U::as_ref);
        if let Some(default) = default {
            prompt = prompt.with_default(default);
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

pub fn required_prompt<T, U>(parameter: Option<T>, default: Option<U>) -> InquireResult<T>
where
    T: FromStr + TextPrompt,
    <T as FromStr>::Err: ToString,
    U: AsRef<str>,
{
    if let Some(value) = parameter {
        Ok(value)
    } else {
        let message = T::name();
        let mut prompt =
            Text::new(&message).with_validator(|input: &str| match input.parse::<T>() {
                Ok(_) => Ok(Validation::Valid),
                Err(error) => Ok(Validation::Invalid(error.into())),
            });
        let help_message = T::help_message();
        if let Some(help_message) = &help_message {
            prompt = prompt.with_help_message(help_message);
        }
        let placeholder = T::placeholder();
        if let Some(placeholder) = &placeholder {
            prompt = prompt.with_placeholder(placeholder);
        }
        let default = default.as_ref().map(U::as_ref);
        if let Some(default) = default {
            prompt = prompt.with_default(default);
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
