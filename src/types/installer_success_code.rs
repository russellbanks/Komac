use crate::prompts::list_prompt::ListPrompt;
use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};
use std::num::NonZeroI64;

#[derive(
    Clone, Debug, Deserialize, Display, Eq, FromStr, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct InstallerSuccessCode(NonZeroI64);

impl ListPrompt for InstallerSuccessCode {
    const MESSAGE: &'static str = "安装成功返回代码:";
    const HELP_MESSAGE: &'static str = "除已知默认值外的其他非零安装程序成功退出代码列表";
    const MAX_ITEMS: u16 = 16;
}
