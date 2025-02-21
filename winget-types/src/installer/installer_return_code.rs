use std::num::NonZeroI64;

use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Copy,
    Debug,
    Display,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    FromStr,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct InstallerReturnCode(NonZeroI64);

pub type InstallerSuccessCode = InstallerReturnCode;
