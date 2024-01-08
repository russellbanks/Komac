use crate::manifests::installer_manifest::{InstallModes, UpgradeBehavior};
use color_eyre::eyre::{Error, Result};
use inquire::{MultiSelect, Select};
use std::collections::BTreeSet;
use std::fmt::Display;
use strum::IntoEnumIterator;

pub trait MultiPrompt {
    const MESSAGE: &'static str;
}

impl MultiPrompt for UpgradeBehavior {
    const MESSAGE: &'static str = "Upgrade behaviour:";
}

impl MultiPrompt for InstallModes {
    const MESSAGE: &'static str = "Install modes:";
}

pub fn radio_prompt<T>() -> Result<T>
where
    T: MultiPrompt + IntoEnumIterator + Display + Ord,
{
    Select::new(T::MESSAGE, T::iter().collect())
        .prompt()
        .map_err(Error::msg)
}

pub fn check_prompt<T>() -> Result<Option<BTreeSet<T>>>
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
        .map_err(Error::msg)
}
