use crate::prompts::prompt::OptionalPrompt;
use crate::types::urls::url::DecodedUrl;
use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

#[derive(Clone, FromStr, Display, Deserialize, Serialize)]
pub struct LicenseUrl(DecodedUrl);

impl OptionalPrompt for LicenseUrl {
    const MESSAGE: &'static str = "License Url:";
    const HELP_MESSAGE: Option<&'static str> = None;
    const PLACEHOLDER: Option<&'static str> = None;
}
