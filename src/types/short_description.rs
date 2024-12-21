use crate::prompts::prompt::Prompt;
use nutype::nutype;

#[nutype(
    validate(len_char_min = 2, len_char_max = 256),
    default = "简短说说这个软件包是做啥的",
    derive(Clone, FromStr, Default, Display, Deserialize, Serialize)
)]
pub struct ShortDescription(String);

impl Prompt for ShortDescription {
    const MESSAGE: &'static str = "简短描述:";
    const HELP_MESSAGE: Option<&'static str> = None;
    const PLACEHOLDER: Option<&'static str> = None;
}
