use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub enum IconTheme {
    Default,
    Light,
    Dark,
    HighContrast,
}
