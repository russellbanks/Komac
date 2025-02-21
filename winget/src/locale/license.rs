use crate::shared::value::{TypeName, Value, ValueConstraints, ValueError};
use compact_str::CompactString;
use derive_more::{Deref, Display, FromStr};
use serde::Serialize;
use serde_with::DeserializeFromStr;

#[derive(
    Clone,
    Debug,
    Default,
    Deref,
    Display,
    Eq,
    PartialEq,
    FromStr,
    Ord,
    PartialOrd,
    Hash,
    Serialize,
    DeserializeFromStr,
)]
pub struct License(Value<3, 512>);

impl License {
    pub fn new<S: Into<CompactString>>(
        value: S,
    ) -> Result<Self, ValueError<Self, { Self::MIN_CHAR_LENGTH }, { Self::MAX_CHAR_LENGTH }>> {
        Value::new(value).map(Self)
    }
}

impl TypeName for License {
    const NAME: &'static str = "License";
}
