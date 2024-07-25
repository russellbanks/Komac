use std::collections::HashMap;
use std::io::{Read, Seek};
use std::str::FromStr;

use camino::Utf8PathBuf;
use color_eyre::eyre::{bail, Result};
use msi::{Language, Package, Select};

use crate::manifests::installer_manifest::Scope;
use crate::types::architecture::Architecture;
use crate::types::language_tag::LanguageTag;

pub struct Msi {
    pub architecture: Architecture,
    pub product_code: String,
    pub upgrade_code: String,
    pub product_name: String,
    pub product_version: String,
    pub manufacturer: String,
    pub product_language: LanguageTag,
    pub all_users: Option<Scope>,
    pub install_location: Option<Utf8PathBuf>,
    pub is_wix: bool,
}

const PRODUCT_CODE: &str = "ProductCode";
const PRODUCT_LANGUAGE: &str = "ProductLanguage";
const PRODUCT_NAME: &str = "ProductName";
const PRODUCT_VERSION: &str = "ProductVersion";
const MANUFACTURER: &str = "Manufacturer";
const UPGRADE_CODE: &str = "UpgradeCode";
const ALL_USERS: &str = "ALLUSERS";
const WIX: &str = "wix";

const INSTALL_DIR: &str = "INSTALLDIR";
const TARGET_DIR: &str = "TARGETDIR";

impl Msi {
    pub fn new<R: Read + Seek>(reader: R) -> Result<Self> {
        let mut msi = Package::open(reader)?;

        let architecture = match msi.summary_info().arch() {
            Some("x64" | "Intel64" | "AMD64") => Architecture::X64,
            Some("Intel") => Architecture::X86,
            Some("Arm64") => Architecture::Arm64,
            Some("Arm") => Architecture::Arm,
            _ => bail!("No architecture was found in the MSI"),
        };

        let mut property_table = Self::get_property_table(&mut msi)?;

        Ok(Self {
            architecture,
            product_code: property_table.remove(PRODUCT_CODE).unwrap(),
            upgrade_code: property_table.remove(UPGRADE_CODE).unwrap(),
            product_name: property_table.remove(PRODUCT_NAME).unwrap(),
            product_version: property_table.remove(PRODUCT_VERSION).unwrap(),
            manufacturer: property_table.remove(MANUFACTURER).unwrap(),
            product_language: LanguageTag::from_str(
                Language::from_code(u16::from_str(
                    property_table.get(PRODUCT_LANGUAGE).unwrap(),
                )?)
                .tag(),
            )?,
            // https://learn.microsoft.com/windows/win32/msi/allusers
            all_users: match property_table
                .remove(ALL_USERS)
                .unwrap_or_default()
                .as_str()
            {
                "1" => Some(Scope::Machine),
                "2" => None, // Installs depending on installation context and user privileges
                _ => Some(Scope::User), // No value or an empty string specifies per-user context
            },
            install_location: Self::find_install_directory(
                &Self::get_directory_table(&mut msi)?,
                &property_table,
            ),
            is_wix: property_table.into_iter().any(|(mut property, mut value)| {
                property.make_ascii_lowercase();
                property.contains(WIX) || {
                    value.make_ascii_lowercase();
                    value.contains(WIX)
                }
            }),
        })
    }

    /// <https://learn.microsoft.com/windows/win32/msi/property-table>
    fn get_property_table<R: Read + Seek>(msi: &mut Package<R>) -> Result<HashMap<String, String>> {
        const PROPERTY: &str = "Property";
        const VALUE: &str = "Value";

        Ok(msi
            .select_rows(Select::table(PROPERTY))?
            .filter_map(|row| match (row[PROPERTY].as_str(), row[VALUE].as_str()) {
                (Some(property), Some(value)) => Some((property.to_owned(), value.to_owned())),
                _ => None,
            })
            .collect::<HashMap<_, _>>())
    }

    /// <https://learn.microsoft.com/windows/win32/msi/directory-table>
    fn get_directory_table<R: Read + Seek>(
        msi: &mut Package<R>,
    ) -> Result<HashMap<String, (Option<String>, String)>> {
        const DIRECTORY: &str = "Directory";
        const DIRECTORY_PARENT: &str = "Directory_Parent";
        const DEFAULT_DIR: &str = "DefaultDir";

        Ok(msi
            .select_rows(Select::table(DIRECTORY))?
            .filter_map(|row| {
                match (
                    row[DIRECTORY].as_str().map(str::to_owned),
                    row[DIRECTORY_PARENT].as_str().map(str::to_owned),
                    row[DEFAULT_DIR].as_str().map(|default_dir| {
                        default_dir
                            .split_once('|')
                            .map_or(default_dir, |(_, long_dir)| long_dir)
                            .to_owned()
                    }),
                ) {
                    (Some(directory), parent, Some(default)) => {
                        Some((directory, (parent, default)))
                    }
                    _ => None,
                }
            })
            .collect::<HashMap<_, _>>())
    }

    fn find_install_directory(
        directory_table: &HashMap<String, (Option<String>, String)>,
        property_table: &HashMap<String, String>,
    ) -> Option<Utf8PathBuf> {
        const WIX_UI_INSTALL_DIR: &str = "WIXUI_INSTALLDIR";

        Self::build_directory(directory_table, INSTALL_DIR, TARGET_DIR).or_else(|| {
            // If `INSTALLDIR` is not in directory table, check value of `WIXUI_INSTALLDIR` property
            property_table
                .get(WIX_UI_INSTALL_DIR)
                .and_then(|wix_install_dir| {
                    Self::build_directory(directory_table, wix_install_dir, TARGET_DIR)
                })
        })
    }

    /// <https://learn.microsoft.com/windows/win32/msi/using-the-directory-table>
    fn build_directory(
        directory_table: &HashMap<String, (Option<String>, String)>,
        current_dir: &str,
        target_dir: &str,
    ) -> Option<Utf8PathBuf> {
        // If the current directory is the target, return an empty path
        if current_dir == target_dir {
            return Some(Utf8PathBuf::new());
        }

        if let Some((Some(parent), default_dir)) = directory_table.get(current_dir) {
            if let Some(mut path) = Self::build_directory(directory_table, parent, target_dir) {
                path.push(Self::get_property_relative_path(current_dir).unwrap_or(default_dir));
                return Some(path);
            }
        }

        None
    }

    fn get_property_relative_path(property: &str) -> Option<&str> {
        const PROGRAM_FILES_64_FOLDER: &str = "ProgramFiles64Folder";
        const RELATIVE_PROGRAM_FILES_64: &str = "%ProgramFiles%";

        const PROGRAM_FILES_FOLDER: &str = "ProgramFilesFolder";
        const RELATIVE_PROGRAM_FILES_32: &str = "%ProgramFiles(x86)%";

        const COMMON_FILES_64_FOLDER: &str = "CommonFiles64Folder";
        const RELATIVE_COMMON_FILES_64: &str = "%CommonProgramFiles%";

        const COMMON_FILES_FOLDER: &str = "CommonFilesFolder";
        const RELATIVE_COMMON_FILES_32: &str = "%CommonProgramFiles(x86)%";

        const APP_DATA_FOLDER: &str = "AppDataFolder";
        const RELATIVE_APP_DATA: &str = "%AppData%";

        const LOCAL_APP_DATA_FOLDER: &str = "LocalAppDataFolder";
        const RELATIVE_LOCAL_APP_DATA: &str = "%LocalAppData%";

        const TEMP_FOLDER: &str = "TempFolder";
        const RELATIVE_TEMP_FOLDER: &str = "%Temp%";

        const WINDOWS_FOLDER: &str = "WindowsFolder";
        const RELATIVE_SYSTEM_ROOT: &str = "%SystemRoot%";

        match property {
            PROGRAM_FILES_64_FOLDER => Some(RELATIVE_PROGRAM_FILES_64),
            PROGRAM_FILES_FOLDER => Some(RELATIVE_PROGRAM_FILES_32),
            COMMON_FILES_64_FOLDER => Some(RELATIVE_COMMON_FILES_64),
            COMMON_FILES_FOLDER => Some(RELATIVE_COMMON_FILES_32),
            APP_DATA_FOLDER => Some(RELATIVE_APP_DATA),
            LOCAL_APP_DATA_FOLDER => Some(RELATIVE_LOCAL_APP_DATA),
            TEMP_FOLDER => Some(RELATIVE_TEMP_FOLDER),
            WINDOWS_FOLDER => Some(RELATIVE_SYSTEM_ROOT),
            _ => None,
        }
    }
}
