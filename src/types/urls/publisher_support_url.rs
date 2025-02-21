use crate::prompts::Prompt;
use crate::prompts::text::TextPrompt;
use crate::types::urls::url::DecodedUrl;
use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

#[derive(Clone, FromStr, Display, Deserialize, Serialize)]
pub struct PublisherSupportUrl(DecodedUrl);

impl Prompt for PublisherSupportUrl {
    const MESSAGE: &'static str = "Publisher Support Url:";
}

impl TextPrompt for PublisherSupportUrl {
    const HELP_MESSAGE: Option<&'static str> = None;
    const PLACEHOLDER: Option<&'static str> = None;
}
