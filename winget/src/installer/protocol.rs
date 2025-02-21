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
    PartialEq,
    FromStr,
    Ord,
    PartialOrd,
    Hash,
    Serialize,
    DeserializeFromStr,
)]
pub struct Protocol(Value<1, 2048>);

impl Protocol {
    pub fn new<S: Into<CompactString>>(
        value: S,
    ) -> Result<Self, ValueError<Self, { Self::MIN_CHAR_LENGTH }, { Self::MAX_CHAR_LENGTH }>> {
        Value::new(value).map(Self)
    }
}

impl TypeName for Protocol {
    const NAME: &'static str = "Protocol";
}

#[cfg(test)]
mod tests {
    use crate::installer::protocol::Protocol;
    use indoc::indoc;

    #[test]
    fn serialize_protocol() {
        assert_eq!(
            serde_yaml::to_string(&Protocol::new("ftp").unwrap()).unwrap(),
            indoc! {"
                ftp
            "}
        );
    }
}
