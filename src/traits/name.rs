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

pub trait Name {
    const NAME: &'static str;
}

macro_rules! impl_names {
    ($( $name:ty => $name_str:literal ),*$(,)?) => {
        $(
            impl Name for $name {
                const NAME: &'static str = $name_str;
            }
        )*
    };
}

impl_names!(
    Author => "Author",
    Channel => "Channel",
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
    PortableCommandAlias => "Portable command alias",
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
