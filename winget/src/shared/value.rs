use compact_str::CompactString;
use derive_more::Display;
use serde::Serialize;
use serde_with::DeserializeFromStr;
use std::fmt;
use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::Deref;
use std::str::FromStr;
use thiserror::Error;

#[derive(
    Clone,
    Debug,
    Default,
    Display,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Serialize,
    DeserializeFromStr,
)]
#[display("{_0}")]
pub struct Value<const MIN: usize, const MAX: usize>(CompactString);

#[derive(Error, Debug, Eq, PartialEq)]
pub enum ValueError<T: TypeName, const MIN: usize, const MAX: usize> {
    TooLong,
    TooShort,
    Phantom(PhantomData<T>),
}

impl<T: TypeName, const MIN: usize, const MAX: usize> Display for ValueError<T, MIN, MAX> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueError::TooLong => {
                write!(f, "{} cannot be more than {MAX} characters long", T::NAME)
            }
            ValueError::TooShort => {
                if MIN == 1 {
                    write!(f, "{} cannot be empty", T::NAME)
                } else {
                    write!(f, "{} must be at least {MIN} characters long", T::NAME)
                }
            }
            ValueError::Phantom(_) => unreachable!(),
        }
    }
}

impl<const MIN: usize, const MAX: usize> ValueConstraints for Value<MIN, MAX> {
    const MIN_CHAR_LENGTH: usize = MIN;
    const MAX_CHAR_LENGTH: usize = MAX;
}

impl<const MIN: usize, const MAX: usize> Value<MIN, MAX> {
    pub fn new<T: TypeName, S: Into<CompactString>>(
        value: S,
    ) -> Result<Self, ValueError<T, MIN, MAX>> {
        let value = value.into();
        match value.chars().count() {
            count if count < MIN => Err(ValueError::TooShort),
            count if count > MAX => Err(ValueError::TooLong),
            _ => Ok(Self(value)),
        }
    }
}

impl<const MIN: usize, const MAX: usize> FromStr for Value<MIN, MAX> {
    type Err = ValueError<Self, MIN, MAX>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new::<Self, _>(s)
    }
}

impl<const MIN: usize, const MAX: usize> TypeName for Value<MIN, MAX> {
    const NAME: &'static str = "Value";
}

pub trait ValueConstraints {
    const MIN_CHAR_LENGTH: usize;
    const MAX_CHAR_LENGTH: usize;
}

// Automatically implement `ValueConstraints` for all types that dereference to a type that
// implements `ValueConstraints`
impl<T, U> ValueConstraints for T
where
    T: Deref<Target = U>,
    U: ValueConstraints,
{
    const MIN_CHAR_LENGTH: usize = U::MIN_CHAR_LENGTH;
    const MAX_CHAR_LENGTH: usize = U::MAX_CHAR_LENGTH;
}

pub trait TypeName {
    const NAME: &'static str;
}
