use crate::prompts::prompt::Prompt;
use crate::types::urls::url::DecodedUrl;
use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

#[derive(Clone, FromStr, Display, Deserialize, Serialize)]
pub struct PackageUrl(DecodedUrl);

impl Prompt for PackageUrl {
    const MESSAGE: &'static str = "Package Url:";
    const HELP_MESSAGE: Option<&'static str> = None;
    const PLACEHOLDER: Option<&'static str> = None;
}
