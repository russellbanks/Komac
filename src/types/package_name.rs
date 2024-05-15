use crate::prompts::prompt::RequiredPrompt;
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
            .and_then(|product_name| Self::new(product_name.trim()).ok())
    }
}

impl RequiredPrompt for PackageName {
    const MESSAGE: &'static str = "Package name:";
    const HELP_MESSAGE: Option<&'static str> = Some("Example: Microsoft Teams");
    const PLACEHOLDER: Option<&'static str> = None;
}
