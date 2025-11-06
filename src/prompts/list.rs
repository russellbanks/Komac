use std::{collections::BTreeSet, fmt::Display, str::FromStr};

use inquire::{Text, validator::Validation};
use winget_types::{
    installer::{Command, FileExtension, InstallerReturnCode, Protocol},
    locale::Tag,
};

use crate::{prompts::handle_inquire_error, traits::Name};

pub trait ListPrompt: Name {
    const PLURAL_NAME: &'static str = Self::NAME;
    const HELP_MESSAGE: &'static str;
    const MAX_ITEMS: u16;
}

impl ListPrompt for InstallerReturnCode {
    const PLURAL_NAME: &'static str = "Installer return codes";
    const HELP_MESSAGE: &'static str = "List of additional non-zero installer success exit codes other than known default values by winget";
    const MAX_ITEMS: u16 = 16;
}

impl ListPrompt for Protocol {
    const PLURAL_NAME: &'static str = "Protocols";
    const HELP_MESSAGE: &'static str =
        "List of protocols the package provides a handler for. Example: http, https";
    const MAX_ITEMS: u16 = 16;
}

impl ListPrompt for FileExtension {
    const PLURAL_NAME: &str = "File extensions";
    const HELP_MESSAGE: &'static str = "List of file extensions the package could support";
    const MAX_ITEMS: u16 = 512;
}

impl ListPrompt for Tag {
    const HELP_MESSAGE: &'static str = "Example: zip, c++, photos, OBS";
    const MAX_ITEMS: u16 = 16;
}

impl ListPrompt for Command {
    const PLURAL_NAME: &'static str = "Commands";
    const HELP_MESSAGE: &'static str = "List of commands or aliases to run the package";
    const MAX_ITEMS: u16 = 16;
}

pub fn list_prompt<T>() -> color_eyre::Result<BTreeSet<T>>
where
    T: FromStr + ListPrompt + Ord,
    <T as FromStr>::Err: Display,
{
    const DELIMITERS: [char; 2] = [' ', ','];
    let items = Text::new(&format!("{}:", T::PLURAL_NAME))
        .with_help_message(T::HELP_MESSAGE)
        .with_validator(|input: &str| {
            let items = input
                .split(|char| DELIMITERS.contains(&char))
                .filter(|str| !str.is_empty())
                .collect::<BTreeSet<_>>();
            let items_len = items.len();
            if items_len > T::MAX_ITEMS as usize {
                return Ok(Validation::Invalid(
                    format!(
                        "There is a maximum of {} items. There were {items_len} provided",
                        T::MAX_ITEMS,
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
