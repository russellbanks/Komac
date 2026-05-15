use std::str::FromStr;

use quick_xml::{Reader, XmlVersion, events::BytesStart};

/// <https://learn.microsoft.com/uwp/schemas/bundlemanifestschema/element-bundle>
#[derive(Debug, Default)]
pub struct Bundle {
    pub identity: Identity,
    pub packages: Vec<Package>,
}

/// <https://learn.microsoft.com/uwp/schemas/bundlemanifestschema/element-identity>
#[derive(Debug, Default)]
pub struct Identity {
    name: String,
    publisher: String,
    version: String,
}

impl Identity {
    pub fn from_event(event: &BytesStart, reader: &mut Reader<&[u8]>) -> quick_xml::Result<Self> {
        debug_assert_eq!(event.local_name().into_inner(), b"Identity");

        let mut identity = Self::default();

        for attribute in event.attributes() {
            let attribute = attribute?;

            match attribute.key.into_inner() {
                b"Name" => {
                    identity.name = attribute
                        .normalized_value(XmlVersion::Implicit1_0)?
                        .into_owned();
                }
                b"Publisher" => {
                    identity.publisher = attribute
                        .normalized_value(XmlVersion::Implicit1_0)?
                        .into_owned();
                }
                b"Version" => {
                    identity.version = attribute
                        .normalized_value(XmlVersion::Implicit1_0)?
                        .into_owned();
                }
                _ => {}
            }
        }

        reader.read_to_end(event.to_end().name())?;

        Ok(identity)
    }

    /// Returns the identity name.
    #[inline]
    pub const fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the identity publisher.
    #[inline]
    pub const fn publisher(&self) -> &str {
        self.publisher.as_str()
    }

    /// Returns the identity version.
    #[expect(unused)]
    #[inline]
    pub const fn version(&self) -> &str {
        self.version.as_str()
    }
}

/// <https://learn.microsoft.com/uwp/schemas/bundlemanifestschema/element-package>
#[derive(Debug, Default)]
pub struct Package {
    /// The type of package in the bundle.
    pub r#type: PackageType,

    /// The file name of the package.
    pub file_name: String,

    /// The offset in bytes into the bundle to the package.
    pub offset: u64,

    /// The size in bytes of the package.
    pub size: u64,

    /// Whether the application in the current package is a stub application.
    pub is_stub: bool,
}

impl Package {
    pub fn from_event(event: &BytesStart, reader: &mut Reader<&[u8]>) -> quick_xml::Result<Self> {
        debug_assert_eq!(event.local_name().into_inner(), b"Package");

        let mut package = Self::default();

        for attribute in event.attributes() {
            let attribute = attribute?;

            match attribute.key.into_inner() {
                b"Type"
                    if let Ok(r#type) = attribute.normalized_value(XmlVersion::Implicit1_0)
                        && let Ok(r#type) = r#type.parse() =>
                {
                    package.r#type = r#type;
                }
                b"FileName" => {
                    package.file_name = attribute
                        .normalized_value(XmlVersion::Implicit1_0)?
                        .into_owned();
                }
                b"Offset"
                    if let Ok(offset) = attribute.normalized_value(XmlVersion::Implicit1_0)
                        && let Ok(offset) = offset.parse() =>
                {
                    package.offset = offset;
                }
                b"Size"
                    if let Ok(size) = attribute.normalized_value(XmlVersion::Implicit1_0)
                        && let Ok(size) = size.parse() =>
                {
                    package.size = size;
                }
                b"IsStub"
                    if let Ok(is_stub) = attribute.normalized_value(XmlVersion::Implicit1_0)
                        && let Ok(is_stub) = is_stub.parse() =>
                {
                    package.is_stub = is_stub;
                }
                _ => {}
            }
        }

        reader.read_to_end(event.to_end().name())?;

        Ok(package)
    }

    #[inline]
    pub const fn is_application(&self) -> bool {
        self.r#type.is_application()
    }

    #[expect(unused)]
    #[inline]
    pub const fn is_resource(&self) -> bool {
        self.r#type.is_resource()
    }

    #[inline]
    pub const fn file_name(&self) -> &str {
        self.file_name.as_str()
    }

    #[inline]
    pub const fn is_stub(&self) -> bool {
        self.is_stub
    }
}

/// <https://learn.microsoft.com/en-gb/uwp/schemas/bundlemanifestschema/element-package#attributes>
#[derive(Debug, Default, PartialEq, Eq)]
pub enum PackageType {
    Application,
    #[default]
    Resource,
}

impl PackageType {
    #[inline]
    pub const fn is_application(&self) -> bool {
        matches!(self, Self::Application)
    }

    #[inline]
    pub const fn is_resource(&self) -> bool {
        matches!(self, Self::Resource)
    }
}

impl FromStr for PackageType {
    type Err = ();

    fn from_str(s: &str) -> color_eyre::Result<Self, Self::Err> {
        match s {
            "application" => Ok(Self::Application),
            "resource" => Ok(Self::Resource),
            _ => Err(()),
        }
    }
}
