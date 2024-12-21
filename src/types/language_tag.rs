use crate::prompts::prompt::Prompt;
use derive_more::{AsRef, Deref, Display, FromStr};
use serde::{Deserialize, Serialize};

#[derive(
    AsRef,
    Clone,
    Debug,
    Deref,
    Display,
    Deserialize,
    Serialize,
    FromStr,
    Eq,
    PartialEq,
    Hash,
    Ord,
    PartialOrd,
)]
pub struct LanguageTag(oxilangtag::LanguageTag<String>);

impl Default for LanguageTag {
    fn default() -> Self { // 默认
        Self::from_str("zh-CN").unwrap()
    }
}

impl Prompt for LanguageTag {
    const MESSAGE: &'static str = "软件包区域:";
    const HELP_MESSAGE: Option<&'static str> = Some("例如: zh-CN (简体中文)");
    const PLACEHOLDER: Option<&'static str> = None;
}
