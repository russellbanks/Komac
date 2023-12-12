use crate::prompts::prompt::OptionalPrompt;
use crate::types::urls::url::Url;
use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

#[derive(Clone, FromStr, Display, Deserialize, Serialize)]
pub struct PackageUrl(Url);

impl OptionalPrompt for PackageUrl {
    const MESSAGE: &'static str = "Package Url:";
    const HELP_MESSAGE: Option<&'static str> = None;
    const PLACEHOLDER: Option<&'static str> = None;
}
