use crate::prompts::prompt::OptionalPrompt;
use crate::types::urls::url::DecodedUrl;
use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

#[derive(Clone, FromStr, Display, Deserialize, Serialize)]
pub struct ReleaseNotesUrl(DecodedUrl);

impl OptionalPrompt for ReleaseNotesUrl {
    const MESSAGE: &'static str = "Release Notes Url:";
    const HELP_MESSAGE: Option<&'static str> = None;
    const PLACEHOLDER: Option<&'static str> = None;
}
