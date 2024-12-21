use crate::prompts::prompt::Prompt;
use crate::types::urls::url::DecodedUrl;
use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

#[derive(Clone, FromStr, Display, Deserialize, Serialize)]
pub struct PublisherSupportUrl(DecodedUrl);

impl Prompt for PublisherSupportUrl {
    const MESSAGE: &'static str = "发布者支持链接:";
    const HELP_MESSAGE: Option<&'static str> = None;
    const PLACEHOLDER: Option<&'static str> = None;
}
