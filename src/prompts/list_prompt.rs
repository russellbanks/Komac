use inquire::validator::Validation;
use inquire::Text;
use std::collections::BTreeSet;
use std::fmt::{Debug, Display};
use std::str::FromStr;

pub trait ListPrompt {
    const MESSAGE: &'static str;
    const HELP_MESSAGE: &'static str;
    const MAX_ITEMS: u16;
}

pub fn list_prompt<T>() -> color_eyre::Result<Option<BTreeSet<T>>>
where
    T: FromStr + ListPrompt + Ord,
    <T as FromStr>::Err: Display + Debug + Sync + Send + 'static,
{
    const DELIMITERS: [char; 2] = [' ', ','];
    let items = Text::new(T::MESSAGE)
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
                if let Err(error) = T::from_str(item) {
                    return Ok(Validation::Invalid(format!("{item}: {error}").into()));
                }
            }
            Ok(Validation::Valid)
        })
        .prompt()?
        .split(|char| DELIMITERS.contains(&char))
        .filter_map(|str| T::from_str(str).ok())
        .collect::<BTreeSet<_>>();
    if items.is_empty() {
        Ok(None)
    } else {
        Ok(Some(items))
    }
}
