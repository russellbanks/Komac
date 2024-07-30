use crate::prompts::prompt::OptionalPrompt;
use crate::types::urls::url::DecodedUrl;
use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

#[derive(Clone, FromStr, Default, Display, Deserialize, Serialize)]
pub struct PublisherSupportUrl(DecodedUrl);

impl OptionalPrompt for PublisherSupportUrl {
    const MESSAGE: &'static str = "Publisher Support Url:";
    const HELP_MESSAGE: Option<&'static str> = None;
    const PLACEHOLDER: Option<&'static str> = None;
}
