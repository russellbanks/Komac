use crate::installer::switches::InstallerSwitch;
use derive_more::{Deref, DerefMut, Display, FromStr, IntoIterator};
use serde_with::{DeserializeFromStr, SerializeDisplay};

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
    pub fn all_users() -> Self {
        "/ALLUSERS".parse().unwrap()
    }

    pub fn current_user() -> Self {
        "/CURRENTUSER".parse().unwrap()
    }
}
