use crate::shared::value::{TypeName, Value, ValueConstraints, ValueError};
use compact_str::CompactString;
use derive_more::{Deref, Display, FromStr};
use serde::Serialize;
use serde_with::DeserializeFromStr;

#[derive(
    Clone,
    Debug,
    Deref,
    Default,
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
pub struct ShortDescription(Value<3, 256>);

impl ShortDescription {
    pub fn new<S: Into<CompactString>>(
        value: S,
    ) -> Result<Self, ValueError<Self, { Self::MIN_CHAR_LENGTH }, { Self::MAX_CHAR_LENGTH }>> {
        Value::new(value).map(Self)
    }
}

impl TypeName for ShortDescription {
    const NAME: &'static str = "Short description";
}
