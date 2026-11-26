use winget_types::{
    LanguageTag, PackageIdentifier, PackageVersion,
    installer::{
        Channel, Command, FileExtension, InstallModes, InstallerReturnCode, PortableCommandAlias,
        Protocol, UpgradeBehavior,
        switches::{CustomSwitch, SilentSwitch, SilentWithProgressSwitch},
    },
    locale::{
        Author, Copyright, Description, InstallationNotes, License, Moniker, PackageName,
        Publisher, ShortDescription, Tag,
    },
    url::{
        CopyrightUrl, LicenseUrl, PackageUrl, PublisherSupportUrl, PublisherUrl, ReleaseNotesUrl,
    },
};

use crate::i18n::fl;

pub trait Name {
    /// Returns the translated name of this manifest field
    fn name() -> String;
}

macro_rules! impl_names {
    ($( $name:ty => $message_id:literal ),*$(,)?) => {
        $(
            impl Name for $name {
                fn name() -> String {
                    fl!($message_id)
                }
            }
        )*
    };
}

impl_names!(
    Author => "field-author",
    Channel => "field-channel",
    Command => "field-command",
    Copyright => "field-copyright",
    CopyrightUrl => "field-copyright-url",
    CustomSwitch => "field-custom-switch",
    Description => "field-description",
    FileExtension => "field-file-extension",
    InstallationNotes => "field-installation-notes",
    InstallerReturnCode => "field-installer-return-code",
    InstallModes => "field-install-modes",
    LanguageTag => "field-language-tag",
    License => "field-license",
    LicenseUrl => "field-license-url",
    Moniker => "field-moniker",
    PackageIdentifier => "field-package-identifier",
    PackageName => "field-package-name",
    PackageUrl => "field-package-url",
    PackageVersion => "field-package-version",
    PortableCommandAlias => "field-portable-command-alias",
    Protocol => "field-protocol",
    Publisher => "field-publisher",
    PublisherSupportUrl => "field-publisher-support-url",
    PublisherUrl => "field-publisher-url",
    ReleaseNotesUrl => "field-release-notes-url",
    ShortDescription => "field-short-description",
    SilentSwitch => "field-silent-switch",
    SilentWithProgressSwitch => "field-silent-with-progress-switch",
    Tag => "field-tag",
    UpgradeBehavior => "field-upgrade-behavior",
);
