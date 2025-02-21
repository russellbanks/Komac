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
pub struct SilentWithProgressSwitch(#[into_iterator(owned, ref, ref_mut)] InstallerSwitch<512>);
