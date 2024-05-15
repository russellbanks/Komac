use crate::prompts::prompt::OptionalPrompt;
use nutype::nutype;
use std::collections::HashMap;

#[nutype(
    validate(len_char_min = 3, len_char_max = 512),
    derive(Clone, FromStr, Display, Deserialize, Serialize)
)]
pub struct Copyright(String);

impl Copyright {
    pub fn get_from_exe(string_map: &HashMap<String, String>) -> Option<Self> {
        string_map
            .get("LegalCopyright")
            .and_then(|legal_copyright| Self::new(legal_copyright.trim()).ok())
    }
}

impl OptionalPrompt for Copyright {
    const MESSAGE: &'static str = "Copyright:";
    const HELP_MESSAGE: Option<&'static str> = Some("Example: Copyright (c) Microsoft Corporation");
    const PLACEHOLDER: Option<&'static str> = None;
}
