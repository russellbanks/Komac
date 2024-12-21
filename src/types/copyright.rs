use crate::prompts::prompt::Prompt;
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
            .and_then(|legal_copyright| Self::try_new(legal_copyright.trim()).ok())
    }
}

impl Prompt for Copyright {
    const MESSAGE: &'static str = "版权:";
    const HELP_MESSAGE: Option<&'static str> = Some("例如: 版权所有 (c) 某某有限公司");
    const PLACEHOLDER: Option<&'static str> = None;
}
