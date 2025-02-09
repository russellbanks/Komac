use crate::prompts::multi::MultiPrompt;
use crate::prompts::prompt::handle_inquire_error;
use bitflags::Flags;
use inquire::error::InquireResult;
use inquire::MultiSelect;
use std::fmt::Display;
use std::ops::BitOr;

pub fn check_prompt<T>() -> InquireResult<Option<T>>
where
    T: MultiPrompt + Flags + Display + BitOr<Output = T> + Copy,
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
