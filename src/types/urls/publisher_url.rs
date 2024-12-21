use crate::prompts::prompt::Prompt;
use crate::types::urls::url::DecodedUrl;
use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

#[derive(Clone, FromStr, Default, Display, Deserialize, Serialize)]
pub struct PublisherUrl(DecodedUrl);

impl Prompt for PublisherUrl {
    const MESSAGE: &'static str = "发布者链接:";
    const HELP_MESSAGE: Option<&'static str> = None;
    const PLACEHOLDER: Option<&'static str> = None;
}
