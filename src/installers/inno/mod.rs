use std::io::Cursor;

use camino::Utf8PathBuf;
use const_format::formatcp;
use inno::{
    error::InnoError,
    header::{Architecture as InnoArchitecture, PrivilegesRequiredOverrides},
};
use msi::Language as CodePageLanguage;
use winget_types::{
    LanguageTag, Sha256String,
    installer::{
        AppsAndFeaturesEntries, AppsAndFeaturesEntry, Architecture as WingetArchitecture,
        ElevationRequirement, InstallationMetadata, Installer, InstallerType, Scope,
        UnsupportedOSArchitecture,
        switches::{CustomSwitch, InstallerSwitches},
    },
    url::DecodedUrl,
};

use super::utils::{
    RELATIVE_APP_DATA, RELATIVE_COMMON_FILES_32, RELATIVE_COMMON_FILES_64, RELATIVE_LOCAL_APP_DATA,
    RELATIVE_PROGRAM_DATA, RELATIVE_PROGRAM_FILES_32, RELATIVE_PROGRAM_FILES_64,
    RELATIVE_SYSTEM_DRIVE, RELATIVE_SYSTEM_ROOT, RELATIVE_WINDOWS_DIR,
};

pub struct Inno {
    pub installers: Vec<Installer>,
}

impl Inno {
    pub fn new(data: &[u8]) -> Result<Self, InnoError> {
        let inno = inno::Inno::new(Cursor::new(data))?;

        let install_dir = inno
            .header
            .default_dir_name()
            .map(str::to_owned)
            .map(to_relative_install_dir)
            .filter(|dir| !dir.contains(['{', '}']));

        let mut installer = Installer {
            locale: inno.primary_language().and_then(|language_entry| {
                CodePageLanguage::from_code(u16::try_from(language_entry.id()).ok()?)
                    .tag()
                    .parse::<LanguageTag>()
                    .ok()
            }),
            architecture: WingetArchitecture::from_inno(inno.header.architectures_allowed()),
            r#type: Some(InstallerType::Inno),
            scope: install_dir
                .as_deref()
                .and_then(Scope::from_install_directory),
            url: DecodedUrl::default(),
            sha_256: Sha256String::default(),
            product_code: inno.header.product_code(),
            unsupported_os_architectures: UnsupportedOSArchitecture::from_inno(
                inno.header.architectures_disallowed(),
            ),
            apps_and_features_entries: if inno.header.uninstall_name().is_some()
                || inno.header.app_publisher().is_some()
                || inno.header.app_version().is_some()
            {
                AppsAndFeaturesEntry::builder()
                    .maybe_display_name(inno.header.uninstall_name())
                    .maybe_publisher(inno.header.app_publisher())
                    .maybe_display_version(inno.header.app_version())
                    .maybe_product_code(inno.header.product_code())
                    .build()
                    .into()
            } else {
                AppsAndFeaturesEntries::new()
            },
            elevation_requirement: inno
                .header
                .privileges_required()
                .to_elevation_requirement(inno.header.privileges_required_overrides_allowed()),
            installation_metadata: InstallationMetadata {
                default_install_location: install_dir.map(Utf8PathBuf::from),
                ..InstallationMetadata::default()
            },
            ..Default::default()
        };

        let installers = if inno
            .header
            .privileges_required_overrides_allowed()
            .is_empty()
        {
            vec![installer]
        } else {
            installer.scope = Some(Scope::Machine);
            let has_scope_switch = inno
                .header
                .privileges_required_overrides_allowed()
                .contains(PrivilegesRequiredOverrides::COMMAND_LINE);
            if has_scope_switch {
                installer.switches = InstallerSwitches::builder()
                    .custom(CustomSwitch::all_users())
                    .build();
            }
            let user_installer = Installer {
                scope: Some(Scope::User),
                switches: InstallerSwitches::builder()
                    .maybe_custom(has_scope_switch.then(CustomSwitch::current_user))
                    .build(),
                installation_metadata: InstallationMetadata::default(),
                ..installer.clone()
            };
            vec![installer, user_installer]
        };

        Ok(Self { installers })
    }
}

trait PrivilegeLevelExt {
    fn to_elevation_requirement(
        &self,
        overrides: PrivilegesRequiredOverrides,
    ) -> Option<ElevationRequirement>;
}

impl PrivilegeLevelExt for inno::header::PrivilegeLevel {
    fn to_elevation_requirement(
        &self,
        overrides: PrivilegesRequiredOverrides,
    ) -> Option<ElevationRequirement> {
        match self {
            Self::Admin | Self::PowerUser => Some(ElevationRequirement::ElevatesSelf),
            _ if !overrides.is_empty() => Some(ElevationRequirement::ElevatesSelf),
            _ => None,
        }
    }
}

pub trait FromInnoArch {
    fn from_inno(value: InnoArchitecture) -> Self;
}

impl FromInnoArch for WingetArchitecture {
    fn from_inno(value: InnoArchitecture) -> Self {
        if value.intersects(
            InnoArchitecture::X64_OS | InnoArchitecture::WIN64 | InnoArchitecture::X64_COMPATIBLE,
        ) {
            Self::X64
        } else if value.intersects(InnoArchitecture::ARM64 | InnoArchitecture::ARM32_COMPATIBLE) {
            Self::Arm64
        } else {
            Self::X86
        }
    }
}

impl FromInnoArch for UnsupportedOSArchitecture {
    fn from_inno(value: InnoArchitecture) -> Self {
        value.iter().fold(
            Self::empty(),
            |unsupported_arch, architecture| match architecture {
                InnoArchitecture::X64_OS
                | InnoArchitecture::WIN64
                | InnoArchitecture::X64_COMPATIBLE => unsupported_arch | Self::X64,
                InnoArchitecture::ARM64 => unsupported_arch | Self::ARM64,
                InnoArchitecture::ARM32_COMPATIBLE => unsupported_arch | Self::ARM,
                InnoArchitecture::X86_OS | InnoArchitecture::X86_COMPATIBLE => {
                    unsupported_arch | Self::X86
                }
                _ => unsupported_arch,
            },
        )
    }
}

pub fn to_relative_install_dir(mut install_dir: String) -> String {
    const WINDOWS: &str = "{win}";
    const SYSTEM: &str = "{sys}";
    const SYSTEM_NATIVE: &str = "{sysnative}";
    const SYSTEM_DRIVE: &str = "{sd}";

    const PROGRAM_FILES: &str = "{commonpf}";
    const PROGRAM_FILES_32: &str = "{commonpf32}";
    const PROGRAM_FILES_64: &str = "{commonpf64}";
    const COMMON_FILES: &str = "{commoncf}";
    const COMMON_FILES_32: &str = "{commoncf32}";
    const COMMON_FILES_64: &str = "{commoncf64}";

    const PROGRAM_FILES_OLD: &str = "{pf}";
    const PROGRAM_FILES_32_OLD: &str = "{pf32}";
    const PROGRAM_FILES_64_OLD: &str = "{pf64}";
    const COMMON_FILES_OLD: &str = "{cf}";
    const COMMON_FILES_32_OLD: &str = "{cf32}";
    const COMMON_FILES_64_OLD: &str = "{cf64}";

    const AUTO_PROGRAM_FILES: &str = "{autopf}";
    const AUTO_PROGRAM_FILES_32: &str = "{autopf32}";
    const AUTO_PROGRAM_FILES_64: &str = "{autopf64}";
    const AUTO_COMMON_FILES: &str = "{autocf}";
    const AUTO_COMMON_FILES_32: &str = "{autocf32}";
    const AUTO_COMMON_FILES_64: &str = "{autocf64}";
    const AUTO_APP_DATA: &str = "{autoappdata}";

    const LOCAL_APP_DATA: &str = "{localappdata}";
    const USER_APP_DATA: &str = "{userappdata}";
    const COMMON_APP_DATA: &str = "{commonappdata}";

    const USER_PROGRAM_FILES: &str = "{userpf}";
    const USER_COMMON_FILES: &str = "{usercf}";

    const RELATIVE_USER_PROGRAM_FILES: &str = formatcp!(r"{RELATIVE_LOCAL_APP_DATA}\Programs");

    const DIRECTORIES: [(&str, &str); 28] = [
        (WINDOWS, RELATIVE_WINDOWS_DIR),
        (SYSTEM, RELATIVE_SYSTEM_ROOT),
        (SYSTEM_NATIVE, RELATIVE_SYSTEM_ROOT),
        (SYSTEM_DRIVE, RELATIVE_SYSTEM_DRIVE),
        (PROGRAM_FILES, RELATIVE_PROGRAM_FILES_64),
        (PROGRAM_FILES_32, RELATIVE_PROGRAM_FILES_32),
        (PROGRAM_FILES_64, RELATIVE_PROGRAM_FILES_64),
        (COMMON_FILES, RELATIVE_COMMON_FILES_64),
        (COMMON_FILES_32, RELATIVE_COMMON_FILES_32),
        (COMMON_FILES_64, RELATIVE_COMMON_FILES_64),
        (PROGRAM_FILES_OLD, RELATIVE_PROGRAM_FILES_64),
        (PROGRAM_FILES_32_OLD, RELATIVE_PROGRAM_FILES_32),
        (PROGRAM_FILES_64_OLD, RELATIVE_PROGRAM_FILES_64),
        (COMMON_FILES_OLD, RELATIVE_COMMON_FILES_64),
        (COMMON_FILES_32_OLD, RELATIVE_COMMON_FILES_32),
        (COMMON_FILES_64_OLD, RELATIVE_COMMON_FILES_64),
        (AUTO_PROGRAM_FILES, RELATIVE_PROGRAM_FILES_64),
        (AUTO_PROGRAM_FILES_32, RELATIVE_PROGRAM_FILES_32),
        (AUTO_PROGRAM_FILES_64, RELATIVE_PROGRAM_FILES_64),
        (AUTO_COMMON_FILES, RELATIVE_COMMON_FILES_64),
        (AUTO_COMMON_FILES_32, RELATIVE_COMMON_FILES_32),
        (AUTO_COMMON_FILES_64, RELATIVE_COMMON_FILES_64),
        (AUTO_APP_DATA, RELATIVE_APP_DATA),
        (LOCAL_APP_DATA, RELATIVE_LOCAL_APP_DATA),
        (USER_APP_DATA, RELATIVE_APP_DATA),
        (COMMON_APP_DATA, RELATIVE_PROGRAM_DATA),
        (USER_PROGRAM_FILES, RELATIVE_USER_PROGRAM_FILES),
        (
            USER_COMMON_FILES,
            formatcp!(r"{RELATIVE_USER_PROGRAM_FILES}\Common"),
        ),
    ];

    for (inno_directory, relative_directory) in DIRECTORIES {
        if let Some(index) = install_dir.find(inno_directory) {
            install_dir.replace_range(index..index + inno_directory.len(), relative_directory);
            break;
        }
    }

    install_dir
}
