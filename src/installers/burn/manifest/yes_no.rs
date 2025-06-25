use std::fmt;

use serde::{Deserialize, Deserializer, de};

pub fn bool_from_yes_no<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    struct YesNoVisitor;

    impl de::Visitor<'_> for YesNoVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("`yes`/`true` or `no`/`false`")
        }

        fn visit_str<E>(self, value: &str) -> Result<bool, E>
        where
            E: de::Error,
        {
            match value {
                "yes" | "true" => Ok(true),
                "no" | "false" => Ok(false),
                _ => Err(de::Error::invalid_value(de::Unexpected::Str(value), &self)),
            }
        }
    }

    deserializer.deserialize_str(YesNoVisitor)
}

#[expect(dead_code)]
#[derive(derive_more::Debug)]
pub enum YesNoButton {
    #[debug("{_0}")]
    YesNo(bool),
    Button,
}

impl Default for YesNoButton {
    fn default() -> Self {
        Self::YesNo(false)
    }
}

impl<'de> Deserialize<'de> for YesNoButton {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct YesNoButtonVisitor;

        impl de::Visitor<'_> for YesNoButtonVisitor {
            type Value = YesNoButton;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("`yes`/`true`, `no`/`false`, or `button`")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    "yes" | "true" => Ok(YesNoButton::YesNo(true)),
                    "no" | "false" => Ok(YesNoButton::YesNo(false)),
                    "button" => Ok(YesNoButton::Button),
                    _ => Err(E::invalid_value(de::Unexpected::Str(value), &self)),
                }
            }
        }

        deserializer.deserialize_string(YesNoButtonVisitor)
    }
}
