use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};
use color_eyre::eyre::Result;
use liblzma::read::XzDecoder;

use crate::exe::inno::version::InnoVersion;

#[derive(Debug, Default)]
pub struct Header {
    app_name: Option<String>,
    app_versioned_name: Option<String>,
    app_id: Option<String>,
    app_copyright: Option<String>,
    app_publisher: Option<String>,
    app_publisher_url: Option<String>,
    app_support_phone: Option<String>,
    app_support_url: Option<String>,
    app_updates_url: Option<String>,
    app_version: Option<String>,
    default_dir_name: Option<String>,
    default_group_name: Option<String>,
    base_filename: Option<String>,
    uninstall_files_dir: Option<String>,
    uninstall_name: Option<String>,
    uninstall_icon: Option<String>,
    app_mutex: Option<String>,
    default_user_name: Option<String>,
    default_user_organisation: Option<String>,
    default_serial: Option<String>,
    app_readme_file: Option<String>,
    app_contact: Option<String>,
    app_comments: Option<String>,
    app_modify_path: Option<String>,
    create_uninstall_registry_key: Option<String>,
    uninstallable: Option<String>,
    close_applications_filter: Option<String>,
    setup_mutex: Option<String>,
    changes_environment: Option<String>,
    changes_associations: Option<String>,
    license_text: Option<String>,
    info_before: Option<String>,
    info_after: Option<String>,
}

impl Header {
    pub fn load<R: Read>(decoder: &mut XzDecoder<R>, version: &InnoVersion) -> Result<Self> {
        let mut header = Header::default();

        header.app_name = binary_string(decoder)?;
        header.app_versioned_name = binary_string(decoder)?;
        if version >= &InnoVersion(1, 3, 0, 0) {
            header.app_id = binary_string(decoder)?;
        }
        header.app_copyright = binary_string(decoder)?;
        if version >= &InnoVersion(1, 3, 0, 0) {
            header.app_publisher = binary_string(decoder)?;
            header.app_publisher_url = binary_string(decoder)?;
        }
        if version >= &InnoVersion(5, 1, 13, 0) {
            header.app_support_phone = binary_string(decoder)?;
        }
        if version >= &InnoVersion(1, 3, 0, 0) {
            header.app_support_url = binary_string(decoder)?;
            header.app_updates_url = binary_string(decoder)?;
            header.app_version = binary_string(decoder)?;
        }
        header.default_dir_name = binary_string(decoder)?;
        header.default_group_name = binary_string(decoder)?;
        header.base_filename = binary_string(decoder)?;
        if version >= &InnoVersion(1, 3, 3, 0) {
            header.uninstall_files_dir = binary_string(decoder)?;
        }
        if version >= &InnoVersion(1, 3, 6, 0) {
            header.uninstall_name = binary_string(decoder)?;
            header.uninstall_icon = binary_string(decoder)?;
        }
        if version >= &InnoVersion(1, 3, 14, 0) {
            header.app_mutex = binary_string(decoder)?;
        }
        if version >= &InnoVersion(3, 0, 0, 0) {
            header.default_user_name = binary_string(decoder)?;
            header.default_user_organisation = binary_string(decoder)?;
        }
        if version >= &InnoVersion(4, 0, 0, 0) {
            header.default_serial = binary_string(decoder)?;
        }
        if version >= &InnoVersion(4, 2, 4, 0) {
            header.app_readme_file = binary_string(decoder)?;
            header.app_contact = binary_string(decoder)?;
            header.app_comments = binary_string(decoder)?;
            header.app_modify_path = binary_string(decoder)?;
        }
        if version >= &InnoVersion(5, 3, 8, 0) {
            header.create_uninstall_registry_key = binary_string(decoder)?;
        }
        if version >= &InnoVersion(5, 3, 10, 0) {
            header.uninstallable = binary_string(decoder)?;
        }
        if version >= &InnoVersion(5, 5, 0, 0) {
            header.close_applications_filter = binary_string(decoder)?;
        }
        if version >= &InnoVersion(5, 5, 6, 0) {
            header.setup_mutex = binary_string(decoder)?;
        }
        if version >= &InnoVersion(5, 6, 1, 0) {
            header.changes_environment = binary_string(decoder)?;
            header.changes_associations = binary_string(decoder)?;
        }
        if version >= &InnoVersion(5, 2, 5, 0) {
            // header.license_text = binary_string(decoder)?;
            // header.info_before = binary_string(decoder)?;
            // header.info_after = binary_string(decoder)?;
        }
        Ok(header)
    }
}

fn binary_string<R: Read>(decoder: &mut XzDecoder<R>) -> Result<Option<String>> {
    let length = decoder.read_u32::<LittleEndian>()?; // Length of UTF-16 string as [u8]
    if length == 0 {
        return Ok(None);
    }
    let mut buf = vec![0; length as usize / size_of::<u16>()]; // Divide by 2 to get [u16]
    decoder.read_u16_into::<LittleEndian>(&mut buf)?;
    Ok(Some(String::from_utf16(&buf)?))
}
