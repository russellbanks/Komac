use std::fmt;

use itertools::Either;
use quick_xml::de::from_str;
use serde::Deserialize;
use tracing::{debug, trace};
use yara_x::mods::{PE, pe::ResourceType::RESOURCE_TYPE_MANIFEST};
use zerocopy::{FromBytes, LE, U16};

use crate::installers::nsis::{state::NsisState, strings::code::NsCode};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
pub struct NsisVersion(pub u8, pub u8, pub u8);

impl NsisVersion {
    #[inline]
    pub const fn v3() -> Self {
        Self(3, 0, 0)
    }

    #[inline]
    pub const fn v2() -> Self {
        Self(2, 0, 0)
    }

    #[inline]
    pub const fn is_v3(self) -> bool {
        self.0 >= 3
    }

    #[inline]
    pub const fn is_v2(self) -> bool {
        !self.is_v3()
    }

    pub fn from_manifest(data: &[u8], pe: &PE) -> Option<Self> {
        #[derive(Deserialize)]
        struct Assembly<'data> {
            #[serde(borrow)]
            description: Description<'data>,
        }

        #[derive(Deserialize)]
        struct Description<'data> {
            #[serde(rename = "$text")]
            inner: &'data str,
        }

        pe.resources
            .iter()
            .find(|resource| resource.type_() == RESOURCE_TYPE_MANIFEST)
            .and_then(|manifest| {
                let offset = manifest.offset() as usize;
                data.get(offset..offset + manifest.length() as usize)
            })
            .and_then(|manifest_bytes| std::str::from_utf8(manifest_bytes).ok())
            .and_then(|manifest| from_str::<Assembly>(manifest).ok())
            .map(|assembly| assembly.description.inner)
            .inspect(|description| debug!(manifest.description = description))
            .and_then(Self::from_text)
    }

    pub fn from_branding_text(state: &NsisState) -> Option<Self> {
        let branding_text = state.get_string(state.language_table.string_offsets[0].get());
        debug!(%branding_text);
        Self::from_text(&branding_text)
    }

    fn from_text(text: &str) -> Option<Self> {
        const NULLSOFT_INSTALL_SYSTEM: &str = "Nullsoft Install System";

        let (text, version) = text.rsplit_once(' ')?;

        if text.trim() != NULLSOFT_INSTALL_SYSTEM {
            return None;
        }

        let mut parts = version
            .trim_start_matches('v')
            .split('.')
            .flat_map(str::parse::<u8>);

        Some(Self(
            parts.next()?,
            parts.next()?,
            parts.next().unwrap_or_default(),
        ))
    }

    pub fn detect(strings_block: &[u8]) -> Self {
        trace!("Detecting version from strings block");

        // The strings block starts with a UTF-16 null byte if it is Unicode
        let unicode = &strings_block[..size_of::<u16>()] == b"\0\0";

        debug!(%unicode);

        let mut nsis3_count = 0;
        let mut nsis2_count = 0;

        let codes = if unicode {
            assert_eq!(strings_block.len() % size_of::<u16>(), 0);

            Either::Left(
                <[U16<LE>]>::ref_from_bytes(strings_block)
                    .unwrap_or_else(|cast_error| unreachable!("{cast_error}"))
                    .windows(2)
                    .filter_map(|window| {
                        (window[0] == U16::ZERO).then(|| window[1].get().try_into().ok())?
                    }),
            )
        } else {
            Either::Right(
                strings_block
                    .windows(2)
                    .filter_map(|window| (window[0] == 0).then_some(window[1])),
            )
        };

        for code in codes {
            if NsCode::is_code(code, Self::v3()) {
                nsis3_count += 1;
            } else if NsCode::is_code(code, Self::v2()) {
                nsis2_count += 1;
            }
        }

        debug!(nsis3_count, nsis2_count);

        if nsis3_count >= nsis2_count {
            Self::v3()
        } else {
            Self::v2()
        }
    }
}

impl Default for NsisVersion {
    fn default() -> Self {
        Self::v3()
    }
}

impl fmt::Display for NsisVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}{}", self.0, self.1, self.2)
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use yara_x::mods::{
        PE,
        pe::{Resource, ResourceType::RESOURCE_TYPE_MANIFEST},
    };

    use crate::installers::nsis::version::NsisVersion;

    #[test]
    fn version_from_manifest() {
        const MANIFEST: &[u8] = indoc! {br#"
            <?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <assembly
                xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
                <assemblyIdentity version="1.0.0.0" processorArchitecture="*" name="Nullsoft.NSIS.exehead" type="win32"/>
                <description>Nullsoft Install System v3.09</description>
            </assembly>
        "#};

        let pe = PE {
            resources: {
                let mut resource = Resource::new();
                resource.set_type(RESOURCE_TYPE_MANIFEST);
                resource.set_offset(0);
                resource.set_length(MANIFEST.len() as u32);
                vec![resource]
            },
            ..PE::default()
        };

        assert_eq!(
            NsisVersion::from_manifest(MANIFEST, &pe).unwrap(),
            NsisVersion(3, 9, 0)
        );
    }

    #[test]
    fn detect_nsis_3() {
        const STRINGS_BLOCK: &[u8; 25] = b"\0\x02Shell\0\x04Skip\0\x01Lang\0\x03Var\0";

        assert_eq!(NsisVersion::detect(STRINGS_BLOCK), NsisVersion::v3());
    }

    #[test]
    fn detect_nsis_2() {
        const STRINGS_BLOCK: &[u8; 25] = b"\0\xFEShell\0\xFCSkip\0\xFFLang\0\xFDVar\0";

        assert_eq!(NsisVersion::detect(STRINGS_BLOCK), NsisVersion::v2());
    }
}
