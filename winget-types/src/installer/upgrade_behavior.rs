use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Display, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub enum UpgradeBehavior {
    Install,
    UninstallPrevious,
    Deny,
}
