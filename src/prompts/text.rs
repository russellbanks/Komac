use crate::prompts::{Prompt, handle_inquire_error};
use inquire::error::InquireResult;
use inquire::validator::Validation;
use inquire::{Confirm, CustomUserError, InquireError, Text};
use std::fmt::{Debug, Display};
use std::str::FromStr;

pub trait TextPrompt: Prompt {
    const HELP_MESSAGE: Option<&'static str>;
    const PLACEHOLDER: Option<&'static str>;
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
