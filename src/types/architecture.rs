use color_eyre::eyre::{bail, Result};
use object::pe::{
    IMAGE_FILE_MACHINE_AMD64, IMAGE_FILE_MACHINE_ARM, IMAGE_FILE_MACHINE_ARM64,
    IMAGE_FILE_MACHINE_ARMNT, IMAGE_FILE_MACHINE_I386, IMAGE_FILE_MACHINE_THUMB,
    IMAGE_FILE_MACHINE_UNKNOWN,
};
use object::read::pe::{ImageNtHeaders, PeFile};
use object::{LittleEndian, ReadRef};
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
    pub fn get_from_exe<'data, Pe, R>(pe: &PeFile<'data, Pe, R>) -> Result<Self>
    where
        Pe: ImageNtHeaders,
        R: ReadRef<'data>,
    {
        Ok(
            match pe.nt_headers().file_header().machine.get(LittleEndian) {
                IMAGE_FILE_MACHINE_AMD64 => Self::X64,
                IMAGE_FILE_MACHINE_I386 => Self::X86,
                IMAGE_FILE_MACHINE_ARM64 => Self::Arm64,
                IMAGE_FILE_MACHINE_ARM | IMAGE_FILE_MACHINE_THUMB | IMAGE_FILE_MACHINE_ARMNT => {
                    Self::Arm
                }
                IMAGE_FILE_MACHINE_UNKNOWN => Self::Neutral,
                machine => bail!("Unexpected architecture: {:04x}", machine),
            },
        )
    }
}
