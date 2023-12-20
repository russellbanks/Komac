use crate::manifests::installer_manifest::{InstallModes, UpgradeBehavior};
use color_eyre::eyre::Error;
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
    type Item = UpgradeBehavior;
    const MESSAGE: &'static str = "Upgrade behaviour";

    fn items() -> Vec<UpgradeBehavior> {
        UpgradeBehavior::iter().collect::<Vec<_>>()
    }
}

impl MultiPrompt for InstallModes {
    type Item = InstallModes;
    const MESSAGE: &'static str = "Install modes";

    fn items() -> Vec<Self::Item> {
        InstallModes::iter().collect::<Vec<_>>()
    }
}

pub fn radio_prompt<T: MultiPrompt>() -> color_eyre::Result<T::Item> {
    Select::new(T::MESSAGE, T::items())
        .prompt()
        .map_err(Error::msg)
}

pub fn check_prompt<T: MultiPrompt>() -> color_eyre::Result<Option<BTreeSet<T::Item>>> {
    MultiSelect::new(T::MESSAGE, T::items())
        .prompt()
        .map(|items| {
            if items.is_empty() {
                None
            } else {
                Some(items.into_iter().collect())
            }
        })
        .map_err(Error::msg)
}
