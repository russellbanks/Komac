use serde::Deserialize;

use super::{Package, bool_from_yes_no};

#[expect(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Chain<'manifest> {
    #[serde(
        rename = "@DisableRollback",
        deserialize_with = "bool_from_yes_no",
        default
    )]
    pub disable_rollback: bool,
    #[serde(
        rename = "@DisableSystemRestore",
        deserialize_with = "bool_from_yes_no",
        default
    )]
    pub disable_system_restore: bool,
    #[serde(
        rename = "@ParallelCache",
        deserialize_with = "bool_from_yes_no",
        default
    )]
    pub parallel_cache: bool,
    #[serde(rename = "$value", borrow)]
    pub packages: Vec<Package<'manifest>>,
}
