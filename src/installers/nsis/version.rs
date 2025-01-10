use crate::installers::nsis::state::NsisState;
use crate::installers::nsis::strings::code::NsCode;
use byteorder::{ByteOrder, LE};
use derive_more::Display;
use itertools::{Either, Itertools};
use quick_xml::de::from_str;
use serde::Deserialize;
use yara_x::mods::pe::ResourceType::RESOURCE_TYPE_MANIFEST;
use yara_x::mods::PE;

#[derive(Debug, Display, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
#[display("{_0}.{_1}{_2}")]
pub struct NsisVersion(pub u8, pub u8, pub u8);

impl NsisVersion {
    pub const _3: Self = Self(3, 0, 0);

    pub const _2: Self = Self(2, 0, 0);

    pub const fn is_v3(self) -> bool {
        self.0 >= 3
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
            .and_then(Self::from_text)
    }

    pub fn from_branding_text(state: &NsisState) -> Option<Self> {
        let branding_text = state.get_string(state.language_table.string_offsets[0].get());
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
        // The strings block starts with a UTF-16 null byte if it is Unicode
        let unicode = &strings_block[..size_of::<u16>()] == b"\0\0";

        let mut nsis3_count = 0;
        let mut nsis2_count = 0;

        let char_size = if unicode {
            size_of::<u16>()
        } else {
            size_of::<u8>()
        };

        let null_indexes = if unicode {
            Either::Left(
                strings_block
                    .chunks_exact(size_of::<u16>())
                    .positions(|chunk| chunk == b"\0\0")
                    .map(|index| index * size_of::<u16>()),
            )
        } else {
            Either::Right(memchr::memchr_iter(0, strings_block))
        };

        let mut pos = char_size;
        for index in null_indexes {
            if index == 0 {
                // Null byte(s) at the start of the string block
                continue;
            }

            let code = strings_block
                .get(pos..index)
                .filter(|string| string.len() >= char_size)
                .and_then(|string| {
                    if unicode {
                        u8::try_from(LE::read_u16(string)).ok()
                    } else {
                        string.first().copied()
                    }
                });

            if let Some(code) = code {
                if NsCode::is_code(code, Self::_3) {
                    nsis3_count += 1;
                } else if NsCode::is_code(code, Self::_2) {
                    nsis2_count += 1;
                }
            }

            pos = index + char_size;
        }

        if nsis3_count >= nsis2_count {
            Self::_3
        } else {
            Self::_2
        }
    }
}

impl Default for NsisVersion {
    fn default() -> Self {
        Self::_3
    }
}

#[cfg(test)]
mod tests {
    use crate::installers::nsis::version::NsisVersion;
    use indoc::indoc;
    use yara_x::mods::pe::Resource;
    use yara_x::mods::pe::ResourceType::RESOURCE_TYPE_MANIFEST;
    use yara_x::mods::PE;

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

        assert_eq!(NsisVersion::detect(STRINGS_BLOCK), NsisVersion::_3);
    }

    #[test]
    fn detect_nsis_2() {
        const STRINGS_BLOCK: &[u8; 25] = b"\0\xFEShell\0\xFCSkip\0\xFFLang\0\xFDVar\0";

        assert_eq!(NsisVersion::detect(STRINGS_BLOCK), NsisVersion::_2);
    }
}
