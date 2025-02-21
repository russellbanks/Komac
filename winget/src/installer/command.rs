use crate::shared::value::{TypeName, Value, ValueConstraints, ValueError};
use compact_str::CompactString;
use derive_more::{Deref, Display, FromStr};
use serde::Serialize;
use serde_with::DeserializeFromStr;

#[derive(
    Clone,
    Debug,
    Deref,
    Display,
    Eq,
    FromStr,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Serialize,
    DeserializeFromStr,
)]
pub struct Command(Value<1, 40>);

impl Command {
    pub fn new<S: Into<CompactString>>(
        value: S,
    ) -> Result<Self, ValueError<Self, { Self::MIN_CHAR_LENGTH }, { Self::MAX_CHAR_LENGTH }>> {
        Value::new(value).map(Self)
    }
}

impl TypeName for Command {
    const NAME: &'static str = "Command";
}
