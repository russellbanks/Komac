use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};
use color_eyre::eyre::Result;
use encoding_rs::{Encoding, UTF_16LE, WINDOWS_1252};

use crate::installers::inno::version::{InnoVersion, KnownVersion};

// https://github.com/jrsoftware/issrc/blob/main/Projects/Src/Shared.Struct.pas
#[derive(Debug, Default)]
pub struct Header {
    pub app_name: Option<String>,
    pub app_versioned_name: Option<String>,
    /// https://jrsoftware.org/ishelp/index.php?topic=setup_appid
    pub app_id: Option<String>,
    pub app_copyright: Option<String>,
    pub app_publisher: Option<String>,
    pub app_publisher_url: Option<String>,
    pub app_support_phone: Option<String>,
    pub app_support_url: Option<String>,
    pub app_updates_url: Option<String>,
    pub app_version: Option<String>,
    pub default_dir_name: Option<String>,
    pub default_group_name: Option<String>,
    pub uninstall_icon_name: Option<String>,
    pub base_filename: Option<String>,
    pub uninstall_files_dir: Option<String>,
    pub uninstall_name: Option<String>,
    pub uninstall_icon: Option<String>,
    pub app_mutex: Option<String>,
    pub default_user_name: Option<String>,
    pub default_user_organisation: Option<String>,
    pub default_serial: Option<String>,
    pub app_readme_file: Option<String>,
    pub app_contact: Option<String>,
    pub app_comments: Option<String>,
    pub app_modify_path: Option<String>,
    pub create_uninstall_registry_key: Option<String>,
    pub uninstallable: Option<String>,
    pub close_applications_filter: Option<String>,
    pub setup_mutex: Option<String>,
    pub changes_environment: Option<String>,
    pub changes_associations: Option<String>,
    pub architectures_allowed: Option<String>,
    pub architectures_install_in_64_bit_mode: Option<String>,
    pub license_text: Option<String>,
    pub info_before: Option<String>,
    pub info_after: Option<String>,
    pub uninstaller_signature: Option<String>,
    pub compiled_code: Option<String>,
    pub language_count: u32,
    pub message_count: u32,
    pub permission_count: u32,
    pub type_count: u32,
    pub component_count: u32,
    pub task_count: u32,
    pub directory_count: u32,
    pub file_count: u32,
    pub data_entry_count: u32,
    pub icon_count: u32,
    pub ini_entry_count: u32,
    pub registry_entry_count: u32,
    pub delete_entry_count: u32,
    pub uninstall_delete_entry_count: u32,
    pub run_entry_count: u32,
    pub uninstall_run_entry_count: u32,
    pub license_size: u32,
    pub info_before_size: u32,
    pub info_after_size: u32,
}

impl Header {
    pub fn load<R: Read>(reader: &mut R, version: &KnownVersion) -> Result<Self> {
        let mut header = Header::default();

        if *version < InnoVersion(1, 3, 0) {
            // Uncompressed size of the setup header
            reader.read_u32::<LittleEndian>()?;
        }

        header.app_name = encoded_string(reader, UTF_16LE)?;
        header.app_versioned_name = encoded_string(reader, UTF_16LE)?;
        if *version >= InnoVersion(1, 3, 0) {
            header.app_id = encoded_string(reader, UTF_16LE)?;
        }
        header.app_copyright = encoded_string(reader, UTF_16LE)?;
        if *version >= InnoVersion(1, 3, 0) {
            header.app_publisher = encoded_string(reader, UTF_16LE)?;
            header.app_publisher_url = encoded_string(reader, UTF_16LE)?;
        }
        if *version >= InnoVersion(5, 1, 13) {
            header.app_support_phone = encoded_string(reader, UTF_16LE)?;
        }
        if *version >= InnoVersion(1, 3, 0) {
            header.app_support_url = encoded_string(reader, UTF_16LE)?;
            header.app_updates_url = encoded_string(reader, UTF_16LE)?;
            header.app_version = encoded_string(reader, UTF_16LE)?;
        }
        header.default_dir_name = encoded_string(reader, UTF_16LE)?;
        header.default_group_name = encoded_string(reader, UTF_16LE)?;
        if *version < InnoVersion(3, 0, 0) {
            header.uninstall_icon_name = encoded_string(reader, WINDOWS_1252)?;
        }
        header.base_filename = encoded_string(reader, UTF_16LE)?;
        if *version >= InnoVersion(1, 3, 0) && *version < InnoVersion(5, 2, 5) {
            header.license_text = encoded_string(reader, WINDOWS_1252)?;
            header.info_before = encoded_string(reader, WINDOWS_1252)?;
            header.info_after = encoded_string(reader, WINDOWS_1252)?;
        }
        if *version >= InnoVersion(1, 3, 3) {
            header.uninstall_files_dir = encoded_string(reader, UTF_16LE)?;
        }
        if *version >= InnoVersion(1, 3, 6) {
            header.uninstall_name = encoded_string(reader, UTF_16LE)?;
            header.uninstall_icon = encoded_string(reader, UTF_16LE)?;
        }
        if *version >= InnoVersion(1, 3, 14) {
            header.app_mutex = encoded_string(reader, UTF_16LE)?;
        }
        if *version >= InnoVersion(3, 0, 0) {
            header.default_user_name = encoded_string(reader, UTF_16LE)?;
            header.default_user_organisation = encoded_string(reader, UTF_16LE)?;
        }
        if *version >= InnoVersion(4, 0, 0) {
            header.default_serial = encoded_string(reader, UTF_16LE)?;
        }
        if *version >= InnoVersion(4, 2, 4) {
            header.app_readme_file = encoded_string(reader, UTF_16LE)?;
            header.app_contact = encoded_string(reader, UTF_16LE)?;
            header.app_comments = encoded_string(reader, UTF_16LE)?;
            header.app_modify_path = encoded_string(reader, UTF_16LE)?;
        }
        if *version >= InnoVersion(5, 3, 8) {
            header.create_uninstall_registry_key = encoded_string(reader, UTF_16LE)?;
        }
        if *version >= InnoVersion(5, 3, 10) {
            header.uninstallable = encoded_string(reader, UTF_16LE)?;
        }
        if *version >= InnoVersion(5, 5, 0) {
            header.close_applications_filter = encoded_string(reader, UTF_16LE)?;
        }
        if *version >= InnoVersion(5, 5, 6) {
            header.setup_mutex = encoded_string(reader, UTF_16LE)?;
        }
        if *version >= InnoVersion(5, 6, 1) {
            header.changes_environment = encoded_string(reader, UTF_16LE)?;
            header.changes_associations = encoded_string(reader, UTF_16LE)?;
        }
        if *version >= InnoVersion(6, 3, 0) {
            header.architectures_allowed = encoded_string(reader, UTF_16LE)?;
            header.architectures_install_in_64_bit_mode = encoded_string(reader, UTF_16LE)?;
        }
        if *version >= InnoVersion(5, 2, 5) {
            header.license_text = encoded_string(reader, WINDOWS_1252)?;
            header.info_before = encoded_string(reader, WINDOWS_1252)?;
            header.info_after = encoded_string(reader, WINDOWS_1252)?;
        }
        if *version >= InnoVersion(5, 2, 1) && *version < InnoVersion(5, 3, 10) {
            header.uninstaller_signature = encoded_string(reader, UTF_16LE)?;
        }
        if *version >= InnoVersion(5, 2, 5) {
            header.compiled_code = encoded_string(reader, UTF_16LE)?;
        }
        /*if *version >= InnoVersion(2, 0, 6) && !version.is_unicode() {
            header.lead_bytes
        }*/
        if *version >= InnoVersion(4, 0, 0) {
            header.language_count = reader.read_u32::<LittleEndian>()?;
        } else if *version >= InnoVersion(2, 0, 1) {
            header.language_count = 1;
        }
        if *version >= InnoVersion(4, 2, 1) {
            header.message_count = reader.read_u32::<LittleEndian>()?;
        }
        if *version >= InnoVersion(4, 1, 0) {
            header.permission_count = reader.read_u32::<LittleEndian>()?;
        }
        if *version >= InnoVersion(2, 0, 0) || version.is_isx() {
            header.type_count = reader.read_u32::<LittleEndian>()?;
            header.component_count = reader.read_u32::<LittleEndian>()?;
        }
        if *version >= InnoVersion(2, 0, 0)
            || (version.is_isx() && *version >= InnoVersion(1, 3, 17))
        {
            header.task_count = reader.read_u32::<LittleEndian>()?;
        }
        header.directory_count = reader.read_u32::<LittleEndian>()?;
        header.file_count = reader.read_u32::<LittleEndian>()?;
        header.data_entry_count = reader.read_u32::<LittleEndian>()?;
        header.icon_count = reader.read_u32::<LittleEndian>()?;
        header.ini_entry_count = reader.read_u32::<LittleEndian>()?;
        header.registry_entry_count = reader.read_u32::<LittleEndian>()?;
        header.delete_entry_count = reader.read_u32::<LittleEndian>()?;
        header.uninstall_delete_entry_count = reader.read_u32::<LittleEndian>()?;
        header.run_entry_count = reader.read_u32::<LittleEndian>()?;
        header.uninstall_run_entry_count = reader.read_u32::<LittleEndian>()?;
        if *version < InnoVersion(1, 3, 0) {
            header.license_size = reader.read_u32::<LittleEndian>()?;
            header.info_before_size = reader.read_u32::<LittleEndian>()?;
            header.info_after_size = reader.read_u32::<LittleEndian>()?;
        }
        Ok(header)
    }
}

fn encoded_string<R: Read>(reader: &mut R, encoding: &'static Encoding) -> Result<Option<String>> {
    let length = reader.read_u32::<LittleEndian>()?;
    if length == 0 {
        return Ok(None);
    }
    let mut buf = vec![0; length as usize];
    reader.read_exact(&mut buf)?;
    Ok(Some(encoding.decode(&buf).0.into_owned()))
}
