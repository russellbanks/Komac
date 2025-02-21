use crate::prompts::Prompt;
use crate::prompts::list::ListPrompt;
use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};
use std::num::NonZeroI64;

#[derive(
    Clone, Debug, Deserialize, Display, Eq, FromStr, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct InstallerSuccessCode(NonZeroI64);

impl Prompt for InstallerSuccessCode {
    const MESSAGE: &'static str = "Installer success codes:";
}

impl ListPrompt for InstallerSuccessCode {
    const HELP_MESSAGE: &'static str = "List of additional non-zero installer success exit codes other than known default values by winget";
    const MAX_ITEMS: u16 = 16;
}
