mod architecture;
mod enums;
mod flag_reader;
mod flags;

use std::io::{Error, ErrorKind, Read, Result};

use crate::installers::inno::encoding::{encoded_string, sized_encoded_string};
use crate::installers::inno::header::architecture::{ArchitectureIdentifiers, StoredArchitecture};
use crate::installers::inno::header::enums::{
    AutoBool, Compression, ImageAlphaFormat, InnoStyle, InstallVerbosity, LanguageDetection,
    LogMode, PrivilegeLevel,
};
use crate::installers::inno::header::flag_reader::FlagReader;
use crate::installers::inno::header::flags::{HeaderFlags, PrivilegesRequiredOverrides};
use crate::installers::inno::version::{InnoVersion, KnownVersion};
use crate::installers::inno::windows_version::WindowsVersionRange;
use bit_set::BitSet;
use byteorder::{ReadBytesExt, LE};
use encoding_rs::{UTF_16LE, WINDOWS_1252};
use zerocopy::TryFromBytes;

macro_rules! enum_value {
    ($reader:expr, $ty:ty) => {{
        let mut buf = [0; size_of::<$ty>()];
        $reader.read_exact(&mut buf)?;
        <$ty>::try_read_from_bytes(&buf)
            .map_err(|error| Error::new(ErrorKind::InvalidData, error.to_string()))
    }};
}

// https://github.com/jrsoftware/issrc/blob/main/Projects/Src/Shared.Struct.pas
#[derive(Debug, Default)]
pub struct Header {
    flags: HeaderFlags,
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
    pub signed_uninstaller_original_size: u32,
    pub signed_uninstaller_header_checksum: u32,
    pub disable_dir_page: AutoBool,
    pub disable_program_group_page: AutoBool,
    pub uninstall_display_size: u64,
}

impl Header {
    pub fn load<R: Read>(reader: &mut R, version: &KnownVersion) -> Result<Self> {
        let mut header = Self::default();

        if *version < InnoVersion(1, 3, 0) {
            // Uncompressed size of the setup header
            reader.read_u32::<LE>()?;
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
                        ArchitectureIdentifiers::X86_COMPATIBLE,
                        ArchitectureIdentifiers::empty(),
                    )
                },
                |architecture| ArchitectureIdentifiers::from_expression(&architecture),
            );
            header.architectures_allowed = allowed;
            header.architectures_disallowed = disallowed;
            header.architectures_install_in_64_bit_mode = encoded_string(reader, UTF_16LE)?
                .map_or(ArchitectureIdentifiers::X86_COMPATIBLE, |architecture| {
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
            let mut buf = [0; 256 / u8::BITS as usize];
            reader.read_exact(&mut buf)?;
            header.lead_bytes = BitSet::from_bytes(&buf);
        }
        if *version >= InnoVersion(4, 0, 0) {
            header.language_count = reader.read_u32::<LE>()?;
        } else if *version >= InnoVersion(2, 0, 1) {
            header.language_count = 1;
        }
        if *version >= InnoVersion(4, 2, 1) {
            header.message_count = reader.read_u32::<LE>()?;
        }
        if *version >= InnoVersion(4, 1, 0) {
            header.permission_count = reader.read_u32::<LE>()?;
        }
        if *version >= InnoVersion(2, 0, 0) || version.is_isx() {
            header.type_count = reader.read_u32::<LE>()?;
            header.component_count = reader.read_u32::<LE>()?;
        }
        if *version >= InnoVersion(2, 0, 0)
            || (version.is_isx() && *version >= InnoVersion(1, 3, 17))
        {
            header.task_count = reader.read_u32::<LE>()?;
        }
        header.directory_count = reader.read_u32::<LE>()?;
        header.file_count = reader.read_u32::<LE>()?;
        header.data_entry_count = reader.read_u32::<LE>()?;
        header.icon_count = reader.read_u32::<LE>()?;
        header.ini_entry_count = reader.read_u32::<LE>()?;
        header.registry_entry_count = reader.read_u32::<LE>()?;
        header.delete_entry_count = reader.read_u32::<LE>()?;
        header.uninstall_delete_entry_count = reader.read_u32::<LE>()?;
        header.run_entry_count = reader.read_u32::<LE>()?;
        header.uninstall_run_entry_count = reader.read_u32::<LE>()?;
        let license_size = if *version < InnoVersion(1, 3, 0) {
            reader.read_u32::<LE>()?
        } else {
            0
        };
        let info_before_size = if *version < InnoVersion(1, 3, 0) {
            reader.read_u32::<LE>()?
        } else {
            0
        };
        let info_after_size = if *version < InnoVersion(1, 3, 0) {
            reader.read_u32::<LE>()?
        } else {
            0
        };
        header.windows_version_range = WindowsVersionRange::load(reader, &version.version)?;
        header.back_color = reader.read_u32::<LE>()?;
        if *version >= InnoVersion(1, 3, 3) {
            header.back_color2 = reader.read_u32::<LE>()?;
        }
        if *version < InnoVersion(5, 5, 7) {
            header.image_back_color = reader.read_u32::<LE>()?;
        }
        if (*version >= InnoVersion(2, 0, 0) && *version < InnoVersion(5, 0, 4)) || version.is_isx()
        {
            header.small_image_back_color = reader.read_u32::<LE>()?;
        }
        if *version >= InnoVersion(6, 0, 0) {
            header.wizard_style = enum_value!(reader, InnoStyle)?;
            header.wizard_resize_percent_x = reader.read_u32::<LE>()?;
            header.wizard_resize_percent_y = reader.read_u32::<LE>()?;
        }
        if *version >= InnoVersion(5, 5, 7) {
            header.image_alpha_format = enum_value!(reader, ImageAlphaFormat)?;
        }
        if *version < InnoVersion(4, 2, 0) {
            let _crc32 = reader.read_u32::<LE>()?;
        } else if *version < InnoVersion(5, 3, 9) {
            let mut md5_buf = [0; 128 / u8::BITS as usize]; // MD5 bit length in bytes
            reader.read_exact(&mut md5_buf)?;
        } else {
            let mut sha1_buf = [0; 160 / u8::BITS as usize]; // SHA1 bit length in bytes
            reader.read_exact(&mut sha1_buf)?;
        }
        if *version >= InnoVersion(4, 2, 2) {
            header.password_salt = Some(password_salt(reader)?);
        }
        if *version >= InnoVersion(4, 0, 0) {
            header.extra_disk_space_required = reader.read_u64::<LE>()?;
            header.slices_per_disk = reader.read_u32::<LE>()?;
        } else {
            header.extra_disk_space_required = u64::from(reader.read_u32::<LE>()?);
            header.slices_per_disk = 1;
        }
        if (*version >= InnoVersion(2, 0, 0) && *version < InnoVersion(5, 0, 0))
            || (version.is_isx() && *version >= InnoVersion(1, 3, 4))
        {
            header.install_verbosity = enum_value!(reader, InstallVerbosity)?;
        }
        if *version >= InnoVersion(1, 3, 0) {
            header.uninstall_log_mode = enum_value!(reader, LogMode)?;
        }
        if *version >= InnoVersion(5, 0, 0) {
            header.uninstall_style = InnoStyle::Modern;
        } else if *version >= InnoVersion(2, 0, 0)
            || (version.is_isx() && *version >= InnoVersion(1, 3, 13))
        {
            header.uninstall_style = enum_value!(reader, InnoStyle)?;
        }
        if *version >= InnoVersion(1, 3, 6) {
            header.dir_exists_warning = enum_value!(reader, AutoBool)?;
        }
        if version.is_isx() && *version >= InnoVersion(2, 0, 10) && *version < InnoVersion(3, 0, 0)
        {
            let _code_line_offset = reader.read_u32::<LE>()?;
        }
        if *version >= InnoVersion(3, 0, 0) && *version < InnoVersion(3, 0, 3) {
            match enum_value!(reader, AutoBool) {
                Ok(AutoBool::Yes) => header.flags |= HeaderFlags::ALWAYS_RESTART,
                Ok(AutoBool::Auto) => {
                    header.flags |= HeaderFlags::RESTART_IF_NEEDED_BY_RUN;
                }
                _ => {}
            }
        }
        if *version >= InnoVersion(3, 0, 4)
            || (version.is_isx() && *version >= InnoVersion(3, 0, 3))
        {
            header.privileges_required = enum_value!(reader, PrivilegeLevel)?;
        }
        if *version >= InnoVersion(5, 7, 0) {
            header.privileges_required_overrides_allowed =
                PrivilegesRequiredOverrides::from_bits_retain(reader.read_u8()?);
        }
        if *version >= InnoVersion(4, 0, 10) {
            header.show_language_dialog = enum_value!(reader, AutoBool)?;
            header.language_detection = enum_value!(reader, LanguageDetection)?;
        }
        if *version >= InnoVersion(5, 3, 9) {
            header.compression = enum_value!(reader, Compression)?;
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
        if *version >= InnoVersion(5, 2, 1) && *version < InnoVersion(5, 3, 10) {
            header.signed_uninstaller_original_size = reader.read_u32::<LE>()?;
            header.signed_uninstaller_header_checksum = reader.read_u32::<LE>()?;
        }
        if *version >= InnoVersion(5, 3, 3) {
            header.disable_dir_page = enum_value!(reader, AutoBool)?;
            header.disable_program_group_page = enum_value!(reader, AutoBool)?;
        }
        if *version >= InnoVersion(5, 5, 0) {
            header.uninstall_display_size = reader.read_u64::<LE>()?;
        } else if *version >= InnoVersion(5, 3, 6) {
            header.uninstall_display_size = u64::from(reader.read_u32::<LE>()?);
        }
        header.flags |= Self::read_flags(reader, version)?;
        if *version < InnoVersion(3, 0, 4) {
            header.privileges_required = PrivilegeLevel::from_header_flags(&header.flags);
        }
        if *version < InnoVersion(4, 0, 10) {
            header.show_language_dialog =
                AutoBool::from_header_flags(&header.flags, HeaderFlags::SHOW_LANGUAGE_DIALOG);
            header.language_detection = LanguageDetection::from_header_flags(&header.flags);
        }
        if *version < InnoVersion(4, 1, 5) {
            header.compression = Compression::from_header_flags(&header.flags);
        }
        if *version < InnoVersion(5, 3, 3) {
            header.disable_dir_page =
                AutoBool::from_header_flags(&header.flags, HeaderFlags::DISABLE_DIR_PAGE);
            header.disable_program_group_page =
                AutoBool::from_header_flags(&header.flags, HeaderFlags::DISABLE_PROGRAM_GROUP_PAGE);
        }
        if *version < InnoVersion(1, 3, 0) {
            header.license_text = sized_encoded_string(reader, license_size, WINDOWS_1252)?;
            header.info_before = sized_encoded_string(reader, info_before_size, WINDOWS_1252)?;
            header.info_after = sized_encoded_string(reader, info_after_size, WINDOWS_1252)?;
        }
        Ok(header)
    }

    fn read_flags<R: Read>(reader: &mut R, version: &KnownVersion) -> Result<HeaderFlags> {
        let mut flags = HeaderFlags::empty();
        let mut flag_reader = FlagReader::new(reader);
        flag_reader.add(HeaderFlags::DISABLE_STARTUP_PROMPT)?;
        if *version < InnoVersion(5, 3, 10) {
            flag_reader.add(HeaderFlags::UNINSTALLABLE)?;
        }
        flag_reader.add(HeaderFlags::CREATE_APP_DIR)?;
        if *version < InnoVersion(5, 3, 3) {
            flag_reader.add(HeaderFlags::DISABLE_DIR_PAGE)?;
        }
        if *version < InnoVersion(1, 3, 6) {
            flag_reader.add(HeaderFlags::DISABLE_DIR_EXISTS_WARNING)?;
        }
        if *version < InnoVersion(5, 3, 3) {
            flag_reader.add(HeaderFlags::DISABLE_PROGRAM_GROUP_PAGE)?;
        }
        flag_reader.add(HeaderFlags::ALLOW_NO_ICONS)?;
        if *version < InnoVersion(3, 0, 0) || *version >= InnoVersion(3, 0, 3) {
            flag_reader.add(HeaderFlags::ALWAYS_RESTART)?;
        }
        if *version < InnoVersion(1, 3, 3) {
            flag_reader.add(HeaderFlags::BACK_SOLID)?;
        }
        flag_reader.add_all([
            HeaderFlags::ALWAYS_USE_PERSONAL_GROUP,
            HeaderFlags::WINDOW_VISIBLE,
            HeaderFlags::WINDOW_SHOW_CAPTION,
            HeaderFlags::WINDOW_RESIZABLE,
            HeaderFlags::WINDOW_START_MAXIMISED,
            HeaderFlags::ENABLED_DIR_DOESNT_EXIST_WARNING,
        ])?;
        if *version < InnoVersion(4, 1, 2) {
            flag_reader.add(HeaderFlags::DISABLE_APPEND_DIR)?;
        }
        flag_reader.add(HeaderFlags::PASSWORD)?;
        if *version >= InnoVersion(1, 2, 6) {
            flag_reader.add(HeaderFlags::ALLOW_ROOT_DIRECTORY)?;
        }
        if *version >= InnoVersion(1, 2, 14) {
            flag_reader.add(HeaderFlags::DISABLE_FINISHED_PAGE)?;
        }
        if *version < InnoVersion(3, 0, 4) {
            flag_reader.add(HeaderFlags::ADMIN_PRIVILEGES_REQUIRED)?;
        }
        if *version < InnoVersion(3, 0, 0) {
            flag_reader.add(HeaderFlags::ALWAYS_CREATE_UNINSTALL_ICON)?;
        }
        if *version < InnoVersion(1, 3, 6) {
            flag_reader.add(HeaderFlags::OVERWRITE_UNINSTALL_REG_ENTRIES)?;
        }
        if *version < InnoVersion(5, 6, 1) {
            flag_reader.add(HeaderFlags::CHANGES_ASSOCIATIONS)?;
        }
        if *version >= InnoVersion(1, 3, 0) && *version < InnoVersion(5, 3, 8) {
            flag_reader.add(HeaderFlags::CREATE_UNINSTALL_REG_KEY)?;
        }
        if *version >= InnoVersion(1, 3, 1) {
            flag_reader.add(HeaderFlags::USE_PREVIOUS_APP_DIR)?;
        }
        if *version >= InnoVersion(1, 3, 3) {
            flag_reader.add(HeaderFlags::BACK_COLOR_HORIZONTAL)?;
        }
        if *version >= InnoVersion(1, 3, 10) {
            flag_reader.add(HeaderFlags::USE_PREVIOUS_GROUP)?;
        }
        if *version >= InnoVersion(1, 3, 20) {
            flag_reader.add(HeaderFlags::UPDATE_UNINSTALL_LOG_APP_NAME)?;
        }
        if *version >= InnoVersion(2, 0, 0)
            || (version.is_isx() && *version >= InnoVersion(1, 3, 10))
        {
            flag_reader.add(HeaderFlags::USE_PREVIOUS_SETUP_TYPE)?;
        }
        if *version >= InnoVersion(2, 0, 0) {
            flag_reader.add_all([
                HeaderFlags::DISABLE_READY_MEMO,
                HeaderFlags::ALWAYS_SHOW_COMPONENTS_LIST,
                HeaderFlags::FLAT_COMPONENTS_LIST,
                HeaderFlags::SHOW_COMPONENT_SIZES,
                HeaderFlags::USE_PREVIOUS_TASKS,
                HeaderFlags::DISABLE_READY_PAGE,
            ])?;
        }
        if *version >= InnoVersion(2, 0, 7) {
            flag_reader.add_all([
                HeaderFlags::ALWAYS_SHOW_DIR_ON_READY_PAGE,
                HeaderFlags::ALWAYS_SHOW_GROUP_ON_READY_PAGE,
            ])?;
        }
        if *version >= InnoVersion(2, 0, 17) && *version < InnoVersion(4, 1, 5) {
            flag_reader.add(HeaderFlags::BZIP_USED)?;
        }
        if *version >= InnoVersion(2, 0, 18) {
            flag_reader.add(HeaderFlags::ALLOW_UNC_PATH)?;
        }
        if *version >= InnoVersion(3, 0, 0) {
            flag_reader.add_all([
                HeaderFlags::USER_INFO_PAGE,
                HeaderFlags::USE_PREVIOUS_USER_INFO,
            ])?;
        }
        if *version >= InnoVersion(3, 0, 1) {
            flag_reader.add(HeaderFlags::UNINSTALL_RESTART_COMPUTER)?;
        }
        if *version >= InnoVersion(3, 0, 3) {
            flag_reader.add(HeaderFlags::RESTART_IF_NEEDED_BY_RUN)?;
        }
        if *version >= InnoVersion(4, 0, 0)
            || (version.is_isx() && *version >= InnoVersion(3, 0, 3))
        {
            flag_reader.add(HeaderFlags::SHOW_TASKS_TREE_LINES)?;
        }
        if *version >= InnoVersion(4, 0, 1) && *version < InnoVersion(4, 0, 10) {
            flag_reader.add(HeaderFlags::DETECT_LANGUAGE_USING_LOCALE)?;
        }
        if *version >= InnoVersion(4, 0, 9) {
            flag_reader.add(HeaderFlags::ALLOW_CANCEL_DURING_INSTALL)?;
        } else {
            flags |= HeaderFlags::ALLOW_CANCEL_DURING_INSTALL;
        }
        if *version >= InnoVersion(4, 1, 3) {
            flag_reader.add(HeaderFlags::WIZARD_IMAGE_STRETCH)?;
        }
        if *version >= InnoVersion(4, 1, 8) {
            flag_reader.add_all([
                HeaderFlags::APPEND_DEFAULT_DIR_NAME,
                HeaderFlags::APPEND_DEFAULT_GROUP_NAME,
            ])?;
        }
        if *version >= InnoVersion(4, 2, 2) {
            flag_reader.add(HeaderFlags::ENCRYPTION_USED)?;
        }
        if *version >= InnoVersion(5, 0, 4) && *version < InnoVersion(5, 6, 1) {
            flag_reader.add(HeaderFlags::CHANGES_ENVIRONMENT)?;
        }
        if *version >= InnoVersion(5, 1, 7) && !version.is_unicode() {
            flag_reader.add(HeaderFlags::SHOW_UNDISPLAYABLE_LANGUAGES)?;
        }
        if *version >= InnoVersion(5, 1, 13) {
            flag_reader.add(HeaderFlags::SETUP_LOGGING)?;
        }
        if *version >= InnoVersion(5, 2, 1) {
            flag_reader.add(HeaderFlags::SIGNED_UNINSTALLER)?;
        }
        if *version >= InnoVersion(5, 3, 8) {
            flag_reader.add(HeaderFlags::USE_PREVIOUS_LANGUAGE)?;
        }
        if *version >= InnoVersion(5, 3, 9) {
            flag_reader.add(HeaderFlags::DISABLE_WELCOME_PAGE)?;
        }
        if *version >= InnoVersion(5, 5, 0) {
            flag_reader.add_all([
                HeaderFlags::CLOSE_APPLICATIONS,
                HeaderFlags::RESTART_APPLICATIONS,
                HeaderFlags::ALLOW_NETWORK_DRIVE,
            ])?;
        } else {
            flags |= HeaderFlags::ALLOW_NETWORK_DRIVE;
        }
        if *version >= InnoVersion(5, 5, 7) {
            flag_reader.add(HeaderFlags::FORCE_CLOSE_APPLICATIONS)?;
        }
        if *version >= InnoVersion(6, 0, 0) {
            flag_reader.add_all([
                HeaderFlags::APP_NAME_HAS_CONSTS,
                HeaderFlags::USE_PREVIOUS_PRIVILEGES,
                HeaderFlags::WIZARD_RESIZABLE,
            ])?;
        }
        if *version >= InnoVersion(6, 3, 0) {
            flag_reader.add(HeaderFlags::UNINSTALL_LOGGING)?;
        }
        flag_reader.finalize().map(|read_flags| flags | read_flags)
    }
}

fn password_salt<R: Read>(reader: &mut R) -> Result<String> {
    const PASSWORD_CHECK_HASH: &str = "PasswordCheckHash";

    let mut password_salt_buf = [0; 8];
    reader.read_exact(&mut password_salt_buf)?;
    let mut password_salt = PASSWORD_CHECK_HASH.to_string();
    password_salt.push_str(
        std::str::from_utf8(&password_salt_buf)
            .map_err(|utf8_err| Error::new(ErrorKind::InvalidData, utf8_err))?,
    );
    Ok(password_salt)
}
