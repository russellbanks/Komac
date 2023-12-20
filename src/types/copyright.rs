use crate::prompts::prompt::OptionalPrompt;
use nutype::nutype;

#[nutype(
    validate(len_char_min = 3, len_char_max = 512),
    derive(Clone, FromStr, Display, Deserialize, Serialize)
)]
pub struct Copyright(String);

impl OptionalPrompt for Copyright {
    const MESSAGE: &'static str = "Copyright:";
    const HELP_MESSAGE: Option<&'static str> = Some("Example: Copyright (c) Microsoft Corporation");
    const PLACEHOLDER: Option<&'static str> = None;
}
