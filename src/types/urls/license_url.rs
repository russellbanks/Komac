use crate::prompts::prompt::OptionalPrompt;
use crate::types::urls::url::Url;
use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

#[derive(Clone, FromStr, Default, Display, Deserialize, Serialize)]
pub struct LicenseUrl(Url);

impl OptionalPrompt for LicenseUrl {
    const MESSAGE: &'static str = "License Url:";
    const HELP_MESSAGE: Option<&'static str> = None;
    const PLACEHOLDER: Option<&'static str> = None;
}
