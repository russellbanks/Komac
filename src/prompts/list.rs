use std::{collections::BTreeSet, fmt::Display, str::FromStr};

use inquire::{Text, validator::Validation};
use winget_types::{
    installer::{Command, FileExtension, InstallerReturnCode, Protocol},
    locale::Tag,
};

use crate::{i18n::fl, prompts::handle_inquire_error, traits::Name};

pub trait ListPrompt: Name {
    const MAX_ITEMS: u16;

    /// Returns the translated name of a list of these fields
    fn plural_name() -> String {
        Self::name()
    }

    /// Returns the translated help message shown below the prompt
    fn help_message() -> String;
}

impl ListPrompt for InstallerReturnCode {
    const MAX_ITEMS: u16 = 16;

    fn plural_name() -> String {
        fl!("field-installer-return-code", "plural")
    }

    fn help_message() -> String {
        fl!("field-installer-return-code", "help")
    }
}

impl ListPrompt for Protocol {
    const MAX_ITEMS: u16 = 16;

    fn plural_name() -> String {
        fl!("field-protocol", "plural")
    }

    fn help_message() -> String {
        fl!("field-protocol", "help")
    }
}

impl ListPrompt for FileExtension {
    const MAX_ITEMS: u16 = 512;

    fn plural_name() -> String {
        fl!("field-file-extension", "plural")
    }

    fn help_message() -> String {
        fl!("field-file-extension", "help")
    }
}

impl ListPrompt for Tag {
    const MAX_ITEMS: u16 = 16;

    fn help_message() -> String {
        fl!("field-tag", "help")
    }
}

impl ListPrompt for Command {
    const MAX_ITEMS: u16 = 16;

    fn plural_name() -> String {
        fl!("field-command", "plural")
    }

    fn help_message() -> String {
        fl!("field-command", "help")
    }
}

pub fn list_prompt<T>() -> color_eyre::Result<BTreeSet<T>>
where
    T: FromStr + ListPrompt + Ord,
    <T as FromStr>::Err: Display,
{
    const DELIMITERS: [char; 2] = [' ', ','];
    let help_message = T::help_message();
    let items = Text::new(&format!("{}:", T::plural_name()))
        .with_help_message(&help_message)
        .with_validator(|input: &str| {
            let items = input
                .split(|char| DELIMITERS.contains(&char))
                .filter(|str| !str.is_empty())
                .collect::<BTreeSet<_>>();
            let items_len = items.len();
            if items_len > T::MAX_ITEMS as usize {
                return Ok(Validation::Invalid(
                    fl!(
                        "prompt-too-many-items",
                        max = T::MAX_ITEMS,
                        count = items_len
                    )
                    .into(),
                ));
            }
            for item in items {
                if let Err(error) = item.parse::<T>() {
                    return Ok(Validation::Invalid(format!("{item}: {error}").into()));
                }
            }
            Ok(Validation::Valid)
        })
        .prompt()
        .map_err(handle_inquire_error)?
        .split(|char| DELIMITERS.contains(&char))
        .flat_map(T::from_str)
        .collect::<BTreeSet<_>>();
    Ok(items)
}
