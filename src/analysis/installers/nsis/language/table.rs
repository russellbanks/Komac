#![allow(unused)]

use std::io::{Error, ErrorKind, Result};

use itertools::Itertools;
use zerocopy::{FromBytes, I32, Immutable, IntoBytes, KnownLayout, LE, U16, U32};

use super::super::header::{
    Header,
    block::{BlockHeaders, BlockType},
};

#[derive(Debug, FromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct LanguageTable {
    id: U16<LE>,
    dialog_offset: U32<LE>,
    right_to_left: U32<LE>,
    string_offsets: [I32<LE>],
}

const EN_US_LANG_CODE: U16<LE> = U16::new(1033);

impl LanguageTable {
    pub fn primary_language<'data>(
        data: &'data [u8],
        header: &Header,
        blocks: &BlockHeaders,
    ) -> Result<&'data Self> {
        BlockType::LangTables
            .get(data, blocks)
            .chunks_exact(header.language_table_size().unsigned_abs() as usize)
            .flat_map(Self::ref_from_bytes)
            .find_or_first(|lang_table| lang_table.id == EN_US_LANG_CODE)
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "No NSIS language table found"))
    }

    #[inline]
    pub const fn id(&self) -> u16 {
        self.id.get()
    }

    #[inline]
    pub const fn dialog_offset(&self) -> u32 {
        self.dialog_offset.get()
    }

    #[inline]
    pub fn right_to_left(&self) -> bool {
        self.right_to_left != U32::ZERO
    }

    pub fn string_offset(&self, index: usize) -> Option<i32> {
        self.string_offsets
            .get(index)
            .copied()
            .filter(|&offset| offset != I32::ZERO)
            .map(I32::get)
    }

    /// Returns an offset to the branding text in the string table.
    #[inline]
    pub fn branding_offset(&self) -> Option<i32> {
        self.string_offset(0)
    }

    /// Returns an offset to the caption text in the string table.
    #[inline]
    pub fn caption_offset(&self) -> Option<i32> {
        self.string_offset(1)
    }

    /// Returns an offset to the application's name in the string table.
    #[inline]
    pub fn name_offset(&self) -> Option<i32> {
        self.string_offset(2)
    }

    /// Returns an offset to the 'space available' text in the string table.
    #[inline]
    pub fn space_available_offset(&self) -> Option<i32> {
        self.string_offset(3)
    }

    /// Returns an offset to the 'space required' text in the string table.
    #[inline]
    pub fn space_required_offset(&self) -> Option<i32> {
        self.string_offset(4)
    }

    /// Returns an offset to the 'can't write' text in the string table.
    #[inline]
    pub fn cant_write_offset(&self) -> Option<i32> {
        self.string_offset(5)
    }

    /// Returns an offset to the 'copy failed' text in the string table.
    #[inline]
    pub fn copy_failed_offset(&self) -> Option<i32> {
        self.string_offset(6)
    }

    /// Returns an offset to the 'copy to' text in the string table.
    #[inline]
    pub fn copy_to_offset(&self) -> Option<i32> {
        self.string_offset(7)
    }

    /// Returns an offset to the 'cannot find symbol' text in the string table.
    #[inline]
    pub fn cannot_find_symbol_offset(&self) -> Option<i32> {
        self.string_offset(8)
    }

    /// Returns an offset to the 'could not load' text in the string table.
    #[inline]
    pub fn could_not_load_offset(&self) -> Option<i32> {
        self.string_offset(9)
    }

    /// Returns an offset to the 'create dir' text in the string table.
    #[inline]
    pub fn create_dir_offset(&self) -> Option<i32> {
        self.string_offset(10)
    }

    /// Returns an offset to the 'create shortcut' text in the string table.
    #[inline]
    pub fn create_shortcut_offset(&self) -> Option<i32> {
        self.string_offset(11)
    }

    /// Returns an offset to the 'created uninstaller' text in the string table.
    #[inline]
    pub fn created_uninstaller_offset(&self) -> Option<i32> {
        self.string_offset(12)
    }

    /// Returns an offset to the 'delete file' text in the string table.
    #[inline]
    pub fn delete_file_offset(&self) -> Option<i32> {
        self.string_offset(13)
    }

    /// Returns an offset to the 'delete on reboot' text in the string table.
    #[inline]
    pub fn delete_on_reboot_offset(&self) -> Option<i32> {
        self.string_offset(14)
    }

    /// Returns an offset to the 'error creating shortcut' text in the string table.
    #[inline]
    pub fn error_creating_shortcut_offset(&self) -> Option<i32> {
        self.string_offset(15)
    }

    /// Returns an offset to the 'error creating' text in the string table.
    #[inline]
    pub fn error_creating_offset(&self) -> Option<i32> {
        self.string_offset(16)
    }

    /// Returns an offset to the 'error decompressing' text in the string table.
    #[inline]
    pub fn error_decompressing_offset(&self) -> Option<i32> {
        self.string_offset(17)
    }

    /// Returns an offset to the 'DLL reg error' text in the string table.
    #[inline]
    pub fn dll_reg_error_offset(&self) -> Option<i32> {
        self.string_offset(18)
    }

    /// Returns an offset to the 'exec shell' text in the string table.
    #[inline]
    pub fn exec_shell_offset(&self) -> Option<i32> {
        self.string_offset(19)
    }

    /// Returns an offset to the 'execute offset' text in the string table.
    #[inline]
    pub fn execute_offset(&self) -> Option<i32> {
        self.string_offset(20)
    }

    /// Returns an offset to the 'extract' text in the string table.
    #[inline]
    pub fn extract_offset(&self) -> Option<i32> {
        self.string_offset(21)
    }

    /// Returns an offset to the 'error writing' text in the string table.
    #[inline]
    pub fn error_writing_offset(&self) -> Option<i32> {
        self.string_offset(22)
    }

    /// Returns an offset to the 'installer corrupted' text in the string table.
    #[inline]
    pub fn installer_corrupted_offset(&self) -> Option<i32> {
        self.string_offset(23)
    }

    /// Returns an offset to the 'No OLE' text in the string table.
    #[inline]
    pub fn no_ole_offset(&self) -> Option<i32> {
        self.string_offset(24)
    }

    /// Returns an offset to the 'output directory' text in the string table.
    #[inline]
    pub fn output_dir_offset(&self) -> Option<i32> {
        self.string_offset(25)
    }

    /// Returns an offset to the 'remove directory' text in the string table.
    #[inline]
    pub fn remove_dir_offset(&self) -> Option<i32> {
        self.string_offset(26)
    }

    /// Returns an offset to the 'rename on reboot' text in the string table.
    #[inline]
    pub fn rename_on_reboot_offset(&self) -> Option<i32> {
        self.string_offset(27)
    }

    /// Returns an offset to the 'rename' text in the string table.
    #[inline]
    pub fn rename_offset(&self) -> Option<i32> {
        self.string_offset(28)
    }

    /// Returns an offset to the 'skipped' text in the string table.
    #[inline]
    pub fn skipped_offset(&self) -> Option<i32> {
        self.string_offset(29)
    }

    /// Returns an offset to the 'copy details' text in the string table.
    #[inline]
    pub fn copy_details_offset(&self) -> Option<i32> {
        self.string_offset(30)
    }

    /// Returns an offset to the 'log install process' text in the string table.
    #[inline]
    pub fn log_install_process_offset(&self) -> Option<i32> {
        self.string_offset(31)
    }

    /// Returns an offset to the 'byte' text in the string table.
    #[inline]
    pub fn byte_offset(&self) -> Option<i32> {
        self.string_offset(32)
    }

    /// Returns an offset to the 'kilo' text in the string table.
    #[inline]
    pub fn kilo_offset(&self) -> Option<i32> {
        self.string_offset(33)
    }

    /// Returns an offset to the 'mega' text in the string table.
    #[inline]
    pub fn mega_offset(&self) -> Option<i32> {
        self.string_offset(34)
    }

    /// Returns an offset to the 'giga' text in the string table.
    #[inline]
    pub fn giga_offset(&self) -> Option<i32> {
        self.string_offset(35)
    }
}

#[cfg(test)]
mod tests {
    use zerocopy::FromBytes;

    use super::{EN_US_LANG_CODE, LanguageTable};

    #[test]
    fn read_language_table() {
        /// Truncated EN-US language table bytes of Standard Notes 3.198.6.
        const LANGUAGE_TABLE_BYTES: [u8; 154] = [
            0x09, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x00,
            0x6E, 0x10, 0x00, 0x00, 0x15, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x2D, 0x12, 0x00, 0x00, 0xDF, 0x13, 0x00, 0x00, 0x06, 0x15, 0x00, 0x00,
            0x8B, 0x17, 0x00, 0x00, 0x5C, 0x19, 0x00, 0x00, 0xF8, 0x1A, 0x00, 0x00, 0xC0, 0x1C,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3A, 0x1E, 0x00, 0x00, 0x9C, 0x20, 0x00, 0x00,
            0x69, 0x23, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x28, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x2A, 0x00, 0x00, 0x23, 0x2B, 0x00, 0x00,
            0x8A, 0x2E, 0x00, 0x00, 0xD2, 0x32, 0x00, 0x00, 0x59, 0x34, 0x00, 0x00, 0xE4, 0x35,
            0x00, 0x00, 0x89, 0x37, 0x00, 0x00, 0x23, 0x3A, 0x00, 0x00, 0x43, 0x3B, 0x00, 0x00,
            0x4E, 0x3C, 0x00, 0x00, 0x57, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let language_table = LanguageTable::ref_from_bytes(&LANGUAGE_TABLE_BYTES).unwrap();

        assert_eq!(language_table.id, EN_US_LANG_CODE);
        assert_eq!(language_table.dialog_offset(), 0);
        assert!(!language_table.right_to_left());

        assert_eq!(language_table.branding_offset(), Some(3_840));
        assert_eq!(language_table.caption_offset(), Some(4_206));
        assert_eq!(language_table.name_offset(), Some(2_325));
        assert_eq!(language_table.space_available_offset(), None);
        assert_eq!(language_table.space_required_offset(), None);
        assert_eq!(language_table.cant_write_offset(), Some(4_653));
        assert_eq!(language_table.copy_failed_offset(), Some(5_087));
        assert_eq!(language_table.copy_to_offset(), Some(5_382));
        assert_eq!(language_table.cannot_find_symbol_offset(), Some(6_027));
        assert_eq!(language_table.could_not_load_offset(), Some(6_492));
        assert_eq!(language_table.create_dir_offset(), Some(6_904));
        assert_eq!(language_table.create_shortcut_offset(), Some(7_360));
        assert_eq!(language_table.created_uninstaller_offset(), None);
        assert_eq!(language_table.delete_file_offset(), Some(7_738));
        assert_eq!(language_table.delete_on_reboot_offset(), Some(8_348));
        assert_eq!(language_table.error_creating_shortcut_offset(), Some(9_065));
        assert_eq!(language_table.error_creating_offset(), None);
        assert_eq!(language_table.error_decompressing_offset(), Some(10_464));
        assert_eq!(language_table.dll_reg_error_offset(), None);
        assert_eq!(language_table.exec_shell_offset(), None);
        assert_eq!(language_table.execute_offset(), Some(10_767));
        assert_eq!(language_table.extract_offset(), Some(11_043));
        assert_eq!(language_table.error_writing_offset(), Some(11_914));
        assert_eq!(language_table.installer_corrupted_offset(), Some(13_010));
        assert_eq!(language_table.no_ole_offset(), Some(13_401));
        assert_eq!(language_table.output_dir_offset(), Some(13_796));
        assert_eq!(language_table.remove_dir_offset(), Some(14_217));
        assert_eq!(language_table.rename_on_reboot_offset(), Some(14_883));
        assert_eq!(language_table.rename_offset(), Some(15_171));
        assert_eq!(language_table.skipped_offset(), Some(15_438));
        assert_eq!(language_table.copy_details_offset(), Some(16_215));
        assert_eq!(language_table.log_install_process_offset(), None);
        assert_eq!(language_table.byte_offset(), None);
        assert_eq!(language_table.kilo_offset(), None);
        assert_eq!(language_table.mega_offset(), None);
        assert_eq!(language_table.giga_offset(), None);
    }
}
