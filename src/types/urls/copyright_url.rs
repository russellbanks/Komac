use crate::prompts::prompt::Prompt;
use crate::types::urls::url::DecodedUrl;
use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

#[derive(Clone, FromStr, Display, Deserialize, Serialize)]
pub struct CopyrightUrl(DecodedUrl);

impl Prompt for CopyrightUrl {
    const MESSAGE: &'static str = "Copyright Url:";
    const HELP_MESSAGE: Option<&'static str> = None;
    const PLACEHOLDER: Option<&'static str> = None;
}
