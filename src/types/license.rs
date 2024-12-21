use crate::prompts::prompt::Prompt;
use nutype::nutype;

#[nutype(
    validate(len_char_min = 3, len_char_max = 512),
    default = "许可证",
    derive(Clone, Default, FromStr, Display, Deserialize, Serialize)
)]
pub struct License(String);

impl Prompt for License {
    const MESSAGE: &'static str = "许可证:";
    const HELP_MESSAGE: Option<&'static str> = Some("例如: MIT, GPL-3.0, Freeware, Proprietary(专有软件)");
    const PLACEHOLDER: Option<&'static str> = None;
}
