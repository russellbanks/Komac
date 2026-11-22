use std::{fmt, marker::PhantomData, str::FromStr};

use serde::{Deserializer, de};

/// Deserializes a value from its string representation using [`FromStr`].
///
/// This is needed for numeric attributes on structs that are `#[serde(flatten)]`ed, as serde
/// buffers flattened attributes as strings, which can't be deserialized directly into numbers.
pub fn deserialize_from_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    struct FromStrVisitor<T>(PhantomData<T>);

    impl<T> de::Visitor<'_> for FromStrVisitor<T>
    where
        T: FromStr,
        T::Err: fmt::Display,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            value.parse().map_err(E::custom)
        }
    }

    deserializer.deserialize_str(FromStrVisitor(PhantomData))
}
