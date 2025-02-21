use crate::prompts::Prompt;
use crate::prompts::text::TextPrompt;
use crate::types::urls::url::DecodedUrl;
use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

#[derive(Clone, FromStr, Display, Deserialize, Serialize)]
pub struct CopyrightUrl(DecodedUrl);

impl Prompt for CopyrightUrl {
    const MESSAGE: &'static str = "Copyright Url:";
}

impl TextPrompt for CopyrightUrl {
    const HELP_MESSAGE: Option<&'static str> = None;
    const PLACEHOLDER: Option<&'static str> = None;
}
