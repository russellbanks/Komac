use std::io;
use std::io::Read;

use crate::installers::inno::header::architecture::{ArchitectureIdentifiers, StoredArchitecture};
use crate::installers::inno::header::enums::{
    AutoBool, Compression, ImageAlphaFormat, InnoStyle, InstallVerbosity, LanguageDetection,
    LogMode, PrivilegeLevel,
};
use crate::installers::inno::header::flags::{HeaderFlags, PrivilegesRequiredOverrides};
use crate::installers::inno::version::{InnoVersion, KnownVersion};
use crate::installers::inno::windows_version::WindowsVersionRange;
use bit_set::BitSet;
use byteorder::{LittleEndian, ReadBytesExt};
use color_eyre::eyre::{eyre, Result};
use encoding_rs::{Encoding, UTF_16LE, WINDOWS_1252};

// https://github.com/jrsoftware/issrc/blob/main/Projects/Src/Shared.Struct.pas
#[derive(Debug, Default)]
pub struct Header {
    header_flags: HeaderFlags,
    pub app_name: Option<String>,
    pub app_versioned_name: Option<String>,
    /// <https://jrsoftware.org/ishelp/index.php?topic=setup_appid>
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
    pub architectures_allowed: ArchitectureIdentifiers,
    pub architectures_disallowed: ArchitectureIdentifiers,
    pub architectures_install_in_64_bit_mode: ArchitectureIdentifiers,
    pub license_text: Option<String>,
    pub info_before: Option<String>,
    pub info_after: Option<String>,
    pub uninstaller_signature: Option<String>,
    pub compiled_code: Option<String>,
    pub lead_bytes: BitSet,
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
    pub windows_version_range: WindowsVersionRange,
    pub back_color: u32,
    pub back_color2: u32,
    pub image_back_color: u32,
    pub small_image_back_color: u32,
    pub wizard_style: InnoStyle,
    pub wizard_resize_percent_x: u32,
    pub wizard_resize_percent_y: u32,
    pub image_alpha_format: ImageAlphaFormat,
    pub password_salt: Option<String>,
    pub extra_disk_space_required: u64,
    pub slices_per_disk: u32,
    pub install_verbosity: InstallVerbosity,
    pub uninstall_log_mode: LogMode,
    pub uninstall_style: InnoStyle,
    pub dir_exists_warning: AutoBool,
    pub privileges_required: PrivilegeLevel,
    pub privileges_required_overrides_allowed: PrivilegesRequiredOverrides,
    pub show_language_dialog: AutoBool,
    pub language_detection: LanguageDetection,
    pub compression: Compression,
}

impl Header {
    pub fn load<R: Read>(reader: &mut R, version: &KnownVersion) -> Result<Self> {
        let mut header = Self::default();

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
        if (*version >= InnoVersion(4, 0, 0) && *version < InnoVersion(5, 2, 5))
            || (version.is_isx() && *version >= InnoVersion(1, 3, 24))
        {
            header.compiled_code = encoded_string(reader, UTF_16LE)?;
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
            let (allowed, disallowed) = encoded_string(reader, UTF_16LE)?.map_or_else(
                || {
                    (
                        ArchitectureIdentifiers::default(),
                        ArchitectureIdentifiers::empty(),
                    )
                },
                |architecture| ArchitectureIdentifiers::from_expression(&architecture),
            );
            header.architectures_allowed = allowed;
            header.architectures_disallowed = disallowed;
            header.architectures_install_in_64_bit_mode = encoded_string(reader, UTF_16LE)?
                .map_or_else(ArchitectureIdentifiers::default, |architecture| {
                    ArchitectureIdentifiers::from_expression(&architecture).0
                });
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
        if *version >= InnoVersion(2, 0, 6) && !version.is_unicode() {
            let mut buf = [0; 256 / 8];
            reader.read_exact(&mut buf)?;
            header.lead_bytes = BitSet::from_bytes(&buf);
        }
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
            let _license_size = reader.read_u32::<LittleEndian>()?;
            let _info_before_size = reader.read_u32::<LittleEndian>()?;
            let _info_after_size = reader.read_u32::<LittleEndian>()?;
        }
        header.windows_version_range = WindowsVersionRange::load(reader, &version.version)?;
        header.back_color = reader.read_u32::<LittleEndian>()?;
        if *version >= InnoVersion(1, 3, 3) {
            header.back_color2 = reader.read_u32::<LittleEndian>()?;
        }
        if *version < InnoVersion(5, 5, 7) {
            header.image_back_color = reader.read_u32::<LittleEndian>()?;
        }
        if (*version >= InnoVersion(2, 0, 0) && *version < InnoVersion(5, 0, 4)) || version.is_isx()
        {
            header.small_image_back_color = reader.read_u32::<LittleEndian>()?;
        }
        if *version >= InnoVersion(6, 0, 0) {
            let wizard_style_value = reader.read_u8()?;
            header.wizard_style = InnoStyle::from_repr(wizard_style_value)
                .ok_or_else(|| eyre!("Unexpected wizard style value: {wizard_style_value}"))?;
            header.wizard_resize_percent_x = reader.read_u32::<LittleEndian>()?;
            header.wizard_resize_percent_y = reader.read_u32::<LittleEndian>()?;
        }
        if *version >= InnoVersion(5, 5, 7) {
            let image_alpha_format = reader.read_u8()?;
            header.image_alpha_format = ImageAlphaFormat::from_repr(image_alpha_format)
                .ok_or_else(|| {
                    eyre!("Unexpected image alpha format value: {image_alpha_format}")
                })?;
        }
        if *version < InnoVersion(4, 2, 0) {
            let _crc32 = reader.read_u32::<LittleEndian>()?;
        } else if *version < InnoVersion(5, 3, 9) {
            let mut md5_buf = [0; 128 / 8]; // MD5 bit length in bytes
            reader.read_exact(&mut md5_buf)?;
        } else {
            let mut sha1_buf = [0; 160 / 8]; // SHA1 bit length in bytes
            reader.read_exact(&mut sha1_buf)?;
        }
        if *version >= InnoVersion(4, 2, 2) {
            header.password_salt = Some(password_salt(reader)?);
        }
        if *version >= InnoVersion(4, 0, 0) {
            header.extra_disk_space_required = reader.read_u64::<LittleEndian>()?;
            header.slices_per_disk = reader.read_u32::<LittleEndian>()?;
        } else {
            header.extra_disk_space_required = u64::from(reader.read_u32::<LittleEndian>()?);
            header.slices_per_disk = 1;
        }
        if (*version >= InnoVersion(2, 0, 0) && *version < InnoVersion(5, 0, 0))
            || (version.is_isx() && *version >= InnoVersion(1, 3, 4))
        {
            let install_verbosity = reader.read_u8()?;
            header.install_verbosity = InstallVerbosity::from_repr(install_verbosity)
                .ok_or_else(|| eyre!("Unexpected install verbosity value: {install_verbosity}"))?;
        }
        if *version >= InnoVersion(1, 3, 0) {
            let uninstall_log_mode = reader.read_u8()?;
            header.uninstall_log_mode =
                LogMode::from_repr(uninstall_log_mode).ok_or_else(|| {
                    eyre!("Unexpected uninstall log mode value: {uninstall_log_mode}")
                })?;
        }
        if *version >= InnoVersion(5, 0, 0) {
            header.uninstall_style = InnoStyle::Modern;
        } else if *version >= InnoVersion(2, 0, 0)
            || (version.is_isx() && *version >= InnoVersion(1, 3, 13))
        {
            let uninstall_style = reader.read_u8()?;
            header.uninstall_style = InnoStyle::from_repr(uninstall_style)
                .ok_or_else(|| eyre!("Unexpected uninstall style value: {uninstall_style}"))?;
        }
        if *version >= InnoVersion(1, 3, 6) {
            let dir_exists_warning = reader.read_u8()?;
            header.dir_exists_warning =
                AutoBool::from_repr(dir_exists_warning).ok_or_else(|| {
                    eyre!("Unexpected dir exists warning value: {dir_exists_warning}")
                })?;
        }
        if version.is_isx() && *version >= InnoVersion(2, 0, 10) && *version < InnoVersion(3, 0, 0)
        {
            let _code_line_offset = reader.read_u32::<LittleEndian>()?;
        }
        if *version >= InnoVersion(3, 0, 0) && *version < InnoVersion(3, 0, 3) {
            match AutoBool::from_repr(reader.read_u8()?) {
                Some(AutoBool::Yes) => header.header_flags |= HeaderFlags::ALWAYS_RESTART,
                Some(AutoBool::Auto) => {
                    header.header_flags |= HeaderFlags::RESTART_IF_NEEDED_BY_RUN;
                }
                _ => {}
            }
        }
        if *version >= InnoVersion(3, 0, 4)
            || (version.is_isx() && *version >= InnoVersion(3, 0, 3))
        {
            let privileges_required = reader.read_u8()?;
            header.privileges_required = PrivilegeLevel::from_repr(privileges_required)
                .ok_or_else(|| {
                    eyre!("Unexpected privileges required value: {privileges_required}")
                })?;
        }
        if *version >= InnoVersion(5, 7, 0) {
            header.privileges_required_overrides_allowed =
                PrivilegesRequiredOverrides::from_bits_retain(reader.read_u8()?);
        }
        if *version >= InnoVersion(4, 0, 10) {
            let show_language_dialog = reader.read_u8()?;
            header.show_language_dialog =
                AutoBool::from_repr(show_language_dialog).ok_or_else(|| {
                    eyre!("Unexpected show language dialog value: {show_language_dialog}")
                })?;
            header.language_detection = LanguageDetection::from_repr(reader.read_u8()?).unwrap();
        }
        if *version >= InnoVersion(5, 3, 9) {
            header.compression = Compression::from_repr(reader.read_u8()?).unwrap();
        }
        if *version >= InnoVersion(5, 1, 0) && *version < InnoVersion(6, 3, 0) {
            header.architectures_allowed =
                StoredArchitecture::from_bits_retain(reader.read_u8()?).to_identifiers();
            header.architectures_install_in_64_bit_mode =
                StoredArchitecture::from_bits_retain(reader.read_u8()?).to_identifiers();
        } else if *version < InnoVersion(5, 1, 0) {
            header.architectures_allowed = StoredArchitecture::all().to_identifiers();
            header.architectures_install_in_64_bit_mode =
                StoredArchitecture::all().to_identifiers();
        }
        Ok(header)
    }
}

fn encoded_string<R: Read>(
    reader: &mut R,
    encoding: &'static Encoding,
) -> io::Result<Option<String>> {
    let length = reader.read_u32::<LittleEndian>()?;
    if length == 0 {
        return Ok(None);
    }
    let mut buf = vec![0; length as usize];
    reader.read_exact(&mut buf)?;
    Ok(Some(encoding.decode(&buf).0.into_owned()))
}

fn password_salt<R: Read>(reader: &mut R) -> io::Result<String> {
    const PASSWORD_CHECK_HASH: &str = "PasswordCheckHash";

    let mut password_salt_buf = [0; 8];
    reader.read_exact(&mut password_salt_buf)?;
    let mut password_salt = PASSWORD_CHECK_HASH.to_string();
    password_salt.push_str(&String::from_utf8_lossy(&password_salt_buf));
    Ok(password_salt)
}
