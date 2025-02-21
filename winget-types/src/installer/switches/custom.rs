use derive_more::{Deref, DerefMut, Display, FromStr, IntoIterator};
use serde_with::{DeserializeFromStr, SerializeDisplay};

use crate::installer::switches::switch::InstallerSwitch;

#[derive(
    Clone,
    Debug,
    Deref,
    DerefMut,
    Display,
    Eq,
    PartialEq,
    FromStr,
    Ord,
    PartialOrd,
    Hash,
    IntoIterator,
    SerializeDisplay,
    DeserializeFromStr,
)]
pub struct CustomSwitch(#[into_iterator(owned, ref, ref_mut)] InstallerSwitch<2048>);

impl CustomSwitch {
    #[must_use]
    pub fn all_users() -> Self {
        "/ALLUSERS".parse().unwrap_or_else(|_| unreachable!())
    }

    #[must_use]
    pub fn current_user() -> Self {
        "/CURRENTUSER".parse().unwrap_or_else(|_| unreachable!())
    }
}
