use crate::manifests::installer_manifest::{InstallModes, UpgradeBehavior};
use crate::prompts::prompt::handle_inquire_error;
use inquire::error::InquireResult;
use inquire::{MultiSelect, Select};
use std::collections::BTreeSet;
use std::fmt::Display;
use strum::IntoEnumIterator;

pub trait MultiPrompt {
    const MESSAGE: &'static str;
}

impl MultiPrompt for UpgradeBehavior {
    const MESSAGE: &'static str = "升级操作:";
}

impl MultiPrompt for InstallModes {
    const MESSAGE: &'static str = "安装模式:";
}

pub fn radio_prompt<T>() -> InquireResult<T>
where
    T: MultiPrompt + IntoEnumIterator + Display,
{
    Select::new(T::MESSAGE, T::iter().collect())
        .prompt()
        .map_err(handle_inquire_error)
}

pub fn check_prompt<T>() -> InquireResult<Option<BTreeSet<T>>>
where
    T: MultiPrompt + IntoEnumIterator + Display + Ord,
{
    MultiSelect::new(T::MESSAGE, T::iter().collect())
        .prompt()
        .map(|items| {
            if items.is_empty() {
                None
            } else {
                Some(BTreeSet::from_iter(items))
            }
        })
        .map_err(handle_inquire_error)
}
