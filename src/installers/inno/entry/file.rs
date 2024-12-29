use crate::installers::inno::encoding::InnoValue;
use crate::installers::inno::entry::condition::Condition;
use crate::installers::inno::enum_value::enum_value::enum_value;
use crate::installers::inno::flag_reader::read_flags::read_flags;
use crate::installers::inno::version::{InnoVersion, KnownVersion};
use crate::installers::inno::windows_version::WindowsVersionRange;
use bitflags::bitflags;
use byteorder::{ReadBytesExt, LE};
use encoding_rs::Encoding;
use std::io::{Read, Result};
use zerocopy::{Immutable, KnownLayout, TryFromBytes};

#[expect(dead_code)]
#[derive(Debug, Default)]
pub struct File {
    source: Option<String>,
    destination: Option<String>,
    install_font_name: Option<String>,
    strong_assembly_name: Option<String>,
    /// Index into the data entry list
    location: u32,
    attributes: u32,
    external_size: u64,
    /// Index into the permission entry list
    permission: i16,
    flags: FileFlags,
    r#type: FileType,
}

impl File {
    pub fn load<R: Read>(
        reader: &mut R,
        codepage: &'static Encoding,
        version: &KnownVersion,
    ) -> Result<Self> {
        if *version < InnoVersion(1, 3, 0) {
            let _uncompressed_size = reader.read_u32::<LE>()?;
        }

        let mut file = Self {
            source: InnoValue::new_string(reader, codepage)?,
            destination: InnoValue::new_string(reader, codepage)?,
            install_font_name: InnoValue::new_string(reader, codepage)?,
            ..Self::default()
        };

        if *version >= InnoVersion(5, 2, 5) {
            file.strong_assembly_name = InnoValue::new_string(reader, codepage)?;
        }

        Condition::load(reader, codepage, version)?;

        WindowsVersionRange::load(reader, version)?;

        file.location = reader.read_u32::<LE>()?;
        file.attributes = reader.read_u32::<LE>()?;
        file.external_size = if *version >= InnoVersion(4, 0, 0) {
            reader.read_u64::<LE>()?
        } else {
            u64::from(reader.read_u32::<LE>()?)
        };

        // TODO: File copy mode

        if *version >= InnoVersion(4, 1, 0) {
            file.permission = reader.read_i16::<LE>()?;
        } else {
            file.permission = -1;
        }

        file.flags = read_flags!(reader,
            [
                FileFlags::CONFIRM_OVERWRITE,
                FileFlags::NEVER_UNINSTALL,
                FileFlags::RESTART_REPLACE,
                FileFlags::DELETE_AFTER_INSTALL,
                FileFlags::REGISTER_SERVER,
                FileFlags::REGISTER_TYPE_LIB,
                FileFlags::SHARED_FILE,
            ],
            if *version < InnoVersion(2, 0, 0) && !version.is_isx() => FileFlags::IS_README_FILE,
            [FileFlags::COMPARE_TIME_STAMP, FileFlags::FONT_IS_NOT_TRUE_TYPE],
            if *version >= InnoVersion(1, 2, 5) => FileFlags::SKIP_IF_SOURCE_DOESNT_EXIST,
            if *version >= InnoVersion(1, 2, 6) => FileFlags::OVERWRITE_READ_ONLY,
            if *version >= InnoVersion(1, 3, 21) => [
                FileFlags::OVERWRITE_SAME_VERSION,
                FileFlags::CUSTOM_DEST_NAME
            ],
            if *version >= InnoVersion(1, 3, 25) => FileFlags::ONLY_IF_DEST_FILE_EXISTS,
            if *version >= InnoVersion(2, 0, 5) => FileFlags::NO_REG_ERROR,
            if *version >= InnoVersion(3, 0, 1) => FileFlags::UNINS_RESTART_DELETE,
            if *version >= InnoVersion(3, 0, 5) => [
                FileFlags::ONLY_IF_DOESNT_EXIST,
                FileFlags::IGNORE_VERSION,
                FileFlags::PROMPT_IF_OLDER,
            ],
            if *version >= InnoVersion(4, 0, 0)
                || (version.is_isx() && *version >= InnoVersion(3, 0, 6)) => FileFlags::DONT_COPY,
            if *version >= InnoVersion(4, 0, 5) => FileFlags::UNINS_REMOVE_READ_ONLY,
            if *version >= InnoVersion(4, 1, 8) => FileFlags::RECURSE_SUB_DIRS_EXTERNAL,
            if *version >= InnoVersion(4, 2, 1) => FileFlags::REPLACE_SAME_VERSION_IF_CONTENTS_DIFFER,
            if *version >= InnoVersion(4, 2, 5) => FileFlags::DONT_VERIFY_CHECKSUM,
            if *version >= InnoVersion(5, 0, 3) => FileFlags::UNINS_NO_SHARED_FILE_PROMPT,
            if *version >= InnoVersion(5, 1, 0) => FileFlags::CREATE_ALL_SUB_DIRS,
            if *version >= InnoVersion(5, 1, 2) => FileFlags::BITS_32, FileFlags::BITS_64,
            if *version >= InnoVersion(5, 2, 0) => [
                FileFlags::EXTERNAL_SIZE_PRESET,
                FileFlags::SET_NTFS_COMPRESSION,
                FileFlags::UNSET_NTFS_COMPRESSION,
            ],
            if *version >= InnoVersion(5, 2, 5) => FileFlags::GAC_INSTALL
        )?;

        file.r#type = enum_value!(reader, FileType)?;

        Ok(file)
    }
}

#[expect(dead_code)]
#[derive(Debug, Default, TryFromBytes, KnownLayout, Immutable)]
#[repr(u8)]
enum FileType {
    #[default]
    UserFile,
    UninstallExe,
    RegSvrExe,
}

bitflags! {
    #[derive(Debug, Default)]
    pub struct FileFlags: u64 {
        const CONFIRM_OVERWRITE = 1 << 0;
        const NEVER_UNINSTALL = 1 << 1;
        const RESTART_REPLACE = 1 << 2;
        const DELETE_AFTER_INSTALL = 1 << 3;
        const REGISTER_SERVER = 1 << 4;
        const REGISTER_TYPE_LIB = 1 << 5;
        const SHARED_FILE = 1 << 6;
        const COMPARE_TIME_STAMP = 1 << 7;
        const FONT_IS_NOT_TRUE_TYPE = 1 << 8;
        const SKIP_IF_SOURCE_DOESNT_EXIST = 1 << 9;
        const OVERWRITE_READ_ONLY = 1 << 10;
        const OVERWRITE_SAME_VERSION = 1 << 11;
        const CUSTOM_DEST_NAME = 1 << 12;
        const ONLY_IF_DEST_FILE_EXISTS = 1 << 13;
        const NO_REG_ERROR = 1 << 14;
        const UNINS_RESTART_DELETE = 1 << 15;
        const ONLY_IF_DOESNT_EXIST = 1 << 16;
        const IGNORE_VERSION = 1 << 17;
        const PROMPT_IF_OLDER = 1 << 18;
        const DONT_COPY = 1 << 19;
        const UNINS_REMOVE_READ_ONLY = 1 << 20;
        const RECURSE_SUB_DIRS_EXTERNAL = 1 << 21;
        const REPLACE_SAME_VERSION_IF_CONTENTS_DIFFER = 1 << 22;
        const DONT_VERIFY_CHECKSUM = 1 << 23;
        const UNINS_NO_SHARED_FILE_PROMPT = 1 << 24;
        const CREATE_ALL_SUB_DIRS = 1 << 25;
        const BITS_32 = 1 << 26;
        const BITS_64 = 1 << 27;
        const EXTERNAL_SIZE_PRESET = 1 << 28;
        const SET_NTFS_COMPRESSION = 1 << 29;
        const UNSET_NTFS_COMPRESSION = 1 << 30;
        const GAC_INSTALL = 1 << 31;
        const IS_README_FILE = 1 << 32;
    }
}
