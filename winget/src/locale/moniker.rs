use crate::locale::tag::Tag;
use crate::shared::value::{TypeName, ValueConstraints, ValueError};
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
    PartialEq,
    FromStr,
    Ord,
    PartialOrd,
    Hash,
    Serialize,
    DeserializeFromStr,
)]
pub struct Moniker(Tag);

impl Moniker {
    pub fn new<S: Into<CompactString>>(
        value: S,
    ) -> Result<Self, ValueError<Self, { Self::MIN_CHAR_LENGTH }, { Self::MAX_CHAR_LENGTH }>> {
        Tag::new(value).map(Self).map_err(|err| match err {
            ValueError::TooLong => ValueError::TooLong,
            ValueError::TooShort => ValueError::TooShort,
            ValueError::Phantom(_) => unreachable!(),
        })
    }
}

impl TypeName for Moniker {
    const NAME: &'static str = "Moniker";
}
