use crate::manifests::installer_manifest::UpgradeBehavior;
use crate::prompts::prompt::handle_inquire_error;
use inquire::error::InquireResult;
use inquire::Select;
use std::fmt::Display;
use strum::IntoEnumIterator;

pub trait MultiPrompt {
    const MESSAGE: &'static str;
}

impl MultiPrompt for UpgradeBehavior {
    const MESSAGE: &'static str = "Upgrade behaviour:";
}

pub fn radio_prompt<T>() -> InquireResult<T>
where
    T: MultiPrompt + IntoEnumIterator + Display,
{
    Select::new(T::MESSAGE, T::iter().collect())
        .prompt()
        .map_err(handle_inquire_error)
}
