use crate::installers::inno::version::InnoVersion;
use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Result};

#[derive(Debug, Default)]
struct Version {
    major: u8,
    minor: u8,
    build: u16,
}

impl Version {
    fn load<R: Read>(reader: &mut R, inno_version: &InnoVersion) -> Result<Self> {
        let mut version = Self::default();
        if *inno_version >= (1, 3, 19) {
            version.build = reader.read_u16::<LE>()?;
        }
        version.minor = reader.read_u8()?;
        version.major = reader.read_u8()?;
        Ok(version)
    }
}

#[derive(Debug, Default)]
struct ServicePack {
    major: u8,
    minor: u8,
}

#[expect(dead_code)]
#[derive(Debug, Default)]
struct WindowsVersion {
    pub win_version: Version,
    pub nt_version: Version,
    pub nt_service_pack: ServicePack,
}

impl WindowsVersion {
    pub fn load<R: Read>(reader: &mut R, version: &InnoVersion) -> Result<Self> {
        let mut windows_version = Self {
            win_version: Version::load(reader, version)?,
            nt_version: Version::load(reader, version)?,
            ..Self::default()
        };

        if *version >= (1, 3, 19) {
            windows_version.nt_service_pack.minor = reader.read_u8()?;
            windows_version.nt_service_pack.major = reader.read_u8()?;
        }

        Ok(windows_version)
    }
}

#[expect(dead_code)]
#[derive(Debug, Default)]
pub struct WindowsVersionRange {
    begin: WindowsVersion,
    end: WindowsVersion,
}

impl WindowsVersionRange {
    pub fn load<R: Read>(reader: &mut R, version: &InnoVersion) -> Result<Self> {
        Ok(Self {
            begin: WindowsVersion::load(reader, version)?,
            end: WindowsVersion::load(reader, version)?,
        })
    }
}
