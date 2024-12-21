use crate::prompts::prompt::Prompt;
use nutype::nutype;
use std::collections::HashMap;

#[nutype(
    validate(len_char_min = 2, len_char_max = 256),
    default = "发布者",
    derive(Clone, Default, Deref, FromStr, Display, Deserialize, Serialize)
)]
pub struct Publisher(String);

impl Publisher {
    pub fn get_from_exe(string_map: &HashMap<String, String>) -> Option<Self> {
        string_map
            .get("CompanyName")
            .and_then(|company_name| Self::try_new(company_name.trim()).ok())
    }
}

impl Prompt for Publisher {
    const MESSAGE: &'static str = "发布者:";
    const HELP_MESSAGE: Option<&'static str> = Some("例如: 某某公司 (发布者不一定是作者)");
    const PLACEHOLDER: Option<&'static str> = None;
}
