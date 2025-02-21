use crate::prompts::Prompt;
use crate::prompts::text::TextPrompt;
use nutype::nutype;
use std::collections::HashMap;

#[nutype(
    validate(len_char_min = 2, len_char_max = 256),
    default = "Package Name",
    derive(Clone, Default, Deref, FromStr, Display, Deserialize, Serialize)
)]
pub struct PackageName(String);

impl PackageName {
    pub fn get_from_exe(string_map: &HashMap<String, String>) -> Option<Self> {
        string_map
            .get("ProductName")
            .and_then(|product_name| Self::try_new(product_name.trim()).ok())
    }
}

impl Prompt for PackageName {
    const MESSAGE: &'static str = "Package name:";
}

impl TextPrompt for PackageName {
    const HELP_MESSAGE: Option<&'static str> = Some("Example: Microsoft Teams");
    const PLACEHOLDER: Option<&'static str> = None;
}
