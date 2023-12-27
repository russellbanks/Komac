use color_eyre::eyre::Error;
use inquire::validator::Validation;
use inquire::Text;
use std::fmt::{Debug, Display};
use std::str::FromStr;

pub trait OptionalPrompt {
    const MESSAGE: &'static str;
    const HELP_MESSAGE: Option<&'static str>;
    const PLACEHOLDER: Option<&'static str>;
}

pub trait RequiredPrompt {
    const MESSAGE: &'static str;
    const HELP_MESSAGE: Option<&'static str>;
    const PLACEHOLDER: Option<&'static str>;
}

pub fn optional_prompt<T>(parameter: Option<T>) -> color_eyre::Result<Option<T>>
where
    T: FromStr + OptionalPrompt,
    <T as FromStr>::Err: Display + Debug + Sync + Send + 'static,
{
    if let Some(value) = parameter {
        Ok(Some(value))
    } else {
        let mut prompt = Text::new(T::MESSAGE).with_validator(|input: &str| {
            if input.is_empty() {
                Ok(Validation::Valid)
            } else {
                let result = T::from_str(input);
                if let Err(error) = result {
                    Ok(Validation::Invalid(error.into()))
                } else {
                    Ok(Validation::Valid)
                }
            }
        });
        if let Some(help_message) = T::HELP_MESSAGE {
            prompt = prompt.with_help_message(help_message);
        }
        if let Some(placeholder) = T::PLACEHOLDER {
            prompt = prompt.with_placeholder(placeholder);
        }
        let result = prompt.prompt()?;
        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(T::from_str(&result).map_err(Error::msg)?))
        }
    }
}

pub fn required_prompt<T>(parameter: Option<T>) -> color_eyre::Result<T>
where
    T: FromStr + RequiredPrompt,
    <T as FromStr>::Err: Display + Debug + Sync + Send + 'static,
{
    if let Some(value) = parameter {
        Ok(value)
    } else {
        let mut prompt = Text::new(T::MESSAGE).with_validator(|input: &str| {
            if let Err(error) = T::from_str(input) {
                Ok(Validation::Invalid(error.into()))
            } else {
                Ok(Validation::Valid)
            }
        });
        if let Some(help_message) = T::HELP_MESSAGE {
            prompt = prompt.with_help_message(help_message);
        }
        if let Some(placeholder) = T::PLACEHOLDER {
            prompt = prompt.with_placeholder(placeholder);
        }
        T::from_str(&prompt.prompt()?).map_err(Error::msg)
    }
}
