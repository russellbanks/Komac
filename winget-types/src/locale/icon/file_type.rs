use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IconFileType {
    Png,
    Jpeg,
    Ico,
}
