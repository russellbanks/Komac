use std::{fmt, fmt::Display, marker::PhantomData, str::FromStr};

use compact_str::CompactString;
use derive_more::{Deref, Display};
use serde::Serialize;
use serde_with::DeserializeFromStr;
use thiserror::Error;

use crate::{
    installer::{
        Command, FileExtension, InstallModes, InstallerReturnCode, Protocol, UpgradeBehavior,
        switches::{CustomSwitch, SilentSwitch, SilentWithProgressSwitch},
    },
    locale::{
        Author, Copyright, Description, InstallationNotes, License, Moniker, PackageName,
        Publisher, ShortDescription, Tag,
    },
    shared::{
        language_tag::LanguageTag,
        package_identifier::PackageIdentifier,
        package_version::PackageVersion,
        url::{
            CopyrightUrl, LicenseUrl, PackageUrl, PublisherSupportUrl, PublisherUrl,
            ReleaseNotesUrl,
        },
    },
};

#[derive(
    Clone,
    Debug,
    Default,
    Deref,
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
pub enum ValueError<T: ValueName, const MIN: usize, const MAX: usize> {
    TooLong,
    TooShort,
    Phantom(PhantomData<T>),
}

impl<T: ValueName, const MIN: usize, const MAX: usize> Display for ValueError<T, MIN, MAX> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLong => {
                write!(f, "{} cannot be more than {MAX} characters long", T::NAME)
            }
            Self::TooShort => {
                if MIN == 1 {
                    write!(f, "{} cannot be empty", T::NAME)
                } else {
                    write!(f, "{} must be at least {MIN} characters long", T::NAME)
                }
            }
            Self::Phantom(_) => unreachable!(),
        }
    }
}

impl<const MIN: usize, const MAX: usize> Value<MIN, MAX> {
    #[expect(clippy::missing_errors_doc)]
    pub fn new<T: ValueName, S: Into<CompactString>>(
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

impl<const MIN: usize, const MAX: usize> ValueName for Value<MIN, MAX> {
    const NAME: &'static str = "Value";
}

#[macro_export]
macro_rules! winget_value {
    ($name:ident, $min:expr, $max:expr) => {
        #[derive(
            Clone,
            Debug,
            Default,
            derive_more::Deref,
            derive_more::Display,
            Eq,
            PartialEq,
            derive_more::FromStr,
            Ord,
            PartialOrd,
            Hash,
            serde::Serialize,
            serde_with::DeserializeFromStr,
        )]
        pub struct $name($crate::shared::value::Value<$min, $max>);

        impl $name {
            #[allow(unused)]
            const MIN_CHAR_LENGTH: usize = $min;

            #[allow(unused)]
            const MAX_CHAR_LENGTH: usize = $max;

            #[expect(clippy::missing_errors_doc)]
            pub fn new<S: Into<compact_str::CompactString>>(
                value: S,
            ) -> Result<Self, $crate::shared::value::ValueError<Self, $min, $max>> {
                $crate::shared::value::Value::new(value).map(Self)
            }
        }
    };
}

pub trait ValueName {
    const NAME: &'static str;
}

macro_rules! value_name {
    ($( $name:ty => $name_str:literal ),*$(,)?) => {
        $(
            impl ValueName for $name {
                const NAME: &'static str = $name_str;
            }
        )*
    };
}

value_name!(
    Author => "Author",
    Command => "Command",
    Copyright => "Copyright",
    CopyrightUrl => "Copyright URL",
    CustomSwitch => "Custom switch",
    Description => "Description",
    FileExtension => "File extension",
    InstallationNotes => "Installation notes",
    InstallerReturnCode => "Installer return code",
    InstallModes => "Install modes",
    LanguageTag => "Language tag",
    License => "License",
    LicenseUrl => "License URL",
    Moniker => "Moniker",
    PackageIdentifier => "Package identifier",
    PackageName => "Package name",
    PackageUrl => "Package URL",
    PackageVersion => "Package version",
    Protocol => "Protocol",
    Publisher => "Publisher",
    PublisherSupportUrl => "Publisher support URL",
    PublisherUrl => "Publisher URL",
    ReleaseNotesUrl => "Release notes URL",
    ShortDescription => "Short description",
    SilentSwitch => "Silent switch",
    SilentWithProgressSwitch => "Silent with progress switch",
    Tag => "Tag",
    UpgradeBehavior => "Upgrade behavior",
);
