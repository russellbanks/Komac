use crate::manifests::installer_manifest::{InstallModes, UpgradeBehavior};
use color_eyre::eyre::{Error, Result};
use inquire::{MultiSelect, Select};
use std::collections::BTreeSet;
use std::fmt::Display;
use strum::IntoEnumIterator;

pub trait MultiPrompt {
    type Item: Display + Ord;
    const MESSAGE: &'static str;
    fn items() -> Vec<Self::Item>;
}

impl MultiPrompt for UpgradeBehavior {
    type Item = Self;
    const MESSAGE: &'static str = "Upgrade behaviour";

    fn items() -> Vec<Self> {
        Self::iter().collect()
    }
}

impl MultiPrompt for InstallModes {
    type Item = Self;
    const MESSAGE: &'static str = "Install modes";

    fn items() -> Vec<Self::Item> {
        Self::iter().collect()
    }
}

pub fn radio_prompt<T: MultiPrompt>() -> Result<T::Item> {
    Select::new(T::MESSAGE, T::items())
        .prompt()
        .map_err(Error::msg)
}

pub fn check_prompt<T: MultiPrompt>() -> Result<Option<BTreeSet<T::Item>>> {
    MultiSelect::new(T::MESSAGE, T::items())
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
