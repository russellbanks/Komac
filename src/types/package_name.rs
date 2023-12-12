use crate::prompts::prompt::RequiredPrompt;
use nutype::nutype;

#[nutype(
    validate(len_char_min = 2, len_char_max = 256),
    default = "Package Name",
    derive(Clone, Default, Deref, FromStr, Display, Deserialize, Serialize)
)]
pub struct PackageName(String);

impl RequiredPrompt for PackageName {
    const MESSAGE: &'static str = "Package name:";
    const HELP_MESSAGE: Option<&'static str> = Some("Example: Microsoft Teams");
    const PLACEHOLDER: Option<&'static str> = None;
}
