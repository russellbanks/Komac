use crate::manifests::installer_manifest::UpgradeBehavior;
use bitflags::Flags;
use inquire::error::InquireResult;
use inquire::{InquireError, MultiSelect, Select};
use std::fmt::Display;
use std::ops::BitOr;
use std::process;
use strum::IntoEnumIterator;

pub mod list;
pub mod text;

pub trait Prompt {
    const MESSAGE: &'static str;
}

impl Prompt for UpgradeBehavior {
    const MESSAGE: &'static str = "Upgrade behaviour:";
}

pub fn radio_prompt<T>() -> InquireResult<T>
where
    T: Prompt + IntoEnumIterator + Display,
{
    Select::new(T::MESSAGE, T::iter().collect())
        .prompt()
        .map_err(handle_inquire_error)
}

pub fn check_prompt<T>() -> InquireResult<Option<T>>
where
    T: Prompt + Flags + Display + BitOr<Output = T> + Copy,
{
    MultiSelect::new(T::MESSAGE, T::all().iter().collect())
        .prompt()
        .map(|items| {
            if items.is_empty() {
                None
            } else {
                Some(items.iter().fold(T::empty(), |flags, flag| flags | *flag))
            }
        })
        .map_err(handle_inquire_error)
}

/// Inquire captures Ctrl+C and returns an error. This will instead exit normally if the prompt is
/// interrupted.
pub fn handle_inquire_error(error: InquireError) -> InquireError {
    if matches!(
        error,
        InquireError::OperationCanceled | InquireError::OperationInterrupted
    ) {
        process::exit(0);
    }
    error
}
