use color_eyre::eyre::{bail, Result};
use exe::{NTHeaders, VecPE, PE};
use serde::{Deserialize, Serialize};
use strum::EnumString;

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    EnumString,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Architecture {
    X86,
    X64,
    Arm,
    Arm64,
    #[default]
    Neutral,
}

impl Architecture {
    pub fn get_from_exe(pe: &VecPE) -> Result<Self> {
        let machine = match pe.get_valid_nt_headers()? {
            NTHeaders::NTHeaders32(nt_header) => nt_header.file_header.machine,
            NTHeaders::NTHeaders64(nt_header) => nt_header.file_header.machine,
        };
        // https://learn.microsoft.com/windows/win32/debug/pe-format#machine-types
        Ok(match machine {
            34404 => Self::X64,           // 0x8664
            332 => Self::X86,             // 0x14c
            43620 => Self::Arm64,         // 0xaa64
            448 | 450 | 452 => Self::Arm, // 0x1c0 | 0x1c2 | 0x1c4
            0 => Self::Neutral,           // 0x0
            _ => bail!("Unknown machine value {:04x}", machine),
        })
    }
}
