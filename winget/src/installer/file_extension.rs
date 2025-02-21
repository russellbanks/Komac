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
pub struct FileExtension(Value<1, 64>);

impl FileExtension {
    pub fn new<S: Into<CompactString>>(
        value: S,
    ) -> Result<Self, ValueError<Self, { Self::MIN_CHAR_LENGTH }, { Self::MAX_CHAR_LENGTH }>> {
        Value::new(value).map(Self)
    }
}

impl TypeName for FileExtension {
    const NAME: &'static str = "File extension";
}
