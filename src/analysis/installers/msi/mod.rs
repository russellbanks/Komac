mod directory_table;
mod property_table;

use std::{
    io::{Error, ErrorKind, Read, Result, Seek},
    str::SplitAsciiWhitespace,
};

use camino::Utf8PathBuf;
use msi::{Language, Package, Select};
use property_table::PropertyTable;
use winget_types::{
    LanguageTag,
    installer::{
        AppsAndFeaturesEntries, AppsAndFeaturesEntry, Architecture, InstallationMetadata,
        Installer, InstallerType, Scope,
    },
};

use crate::{
    analysis::{installers::msi::directory_table::DirectoryTable, r#trait::Installers},
    traits::AsciiExt,
};

const PROPERTY: &str = "Property";
const CONTROL: &str = "Control";

const ALL_USERS: &str = "ALLUSERS";
const INSTALL_DIR: &str = "INSTALLDIR";
const TARGET_DIR: &str = "TARGETDIR";

pub struct Msi {
    pub architecture: Architecture,
    pub all_users: Option<Scope>,
    pub property_table: PropertyTable,
    pub directory_table: DirectoryTable,
    pub creating_application: Option<String>,
    pub comments: Option<String>,
}

impl Msi {
    pub fn new<R: Read + Seek>(reader: R) -> Result<Self> {
        let mut msi = Package::open(reader)?;

        let architecture = match msi.summary_info().arch() {
            Some("x64" | "Intel64" | "AMD64") => Architecture::X64,
            Some("Intel") | None => Architecture::X86,
            Some("Arm64") => Architecture::Arm64,
            Some("Arm") => Architecture::Arm,
            Some(arch) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!(r#"Unknown MSI architecture: "{arch}""#),
                ));
            }
        };

        let property_table = PropertyTable::new(&mut msi)?;

        // https://learn.microsoft.com/windows/win32/msi/allusers
        let all_users = match property_table.get(ALL_USERS) {
            Some("1") => Some(Scope::Machine),
            Some("2") => None, // Installs depending on installation context and user privileges
            Some("") => Some(Scope::User), // An empty string specifies per-user context
            _ => {
                if msi
                    .select_rows(Select::table(CONTROL).columns(&[PROPERTY]))
                    .is_ok_and(|mut rows| rows.any(|row| row[0].as_str() == Some(ALL_USERS)))
                {
                    // ALLUSERS could be changed at runtime
                    None
                } else {
                    // No value or control specifies per-user context
                    Some(Scope::User)
                }
            }
        };

        let directory_table = DirectoryTable::new(&mut msi)?;
        let summary_info = msi.summary_info();

        Ok(Self {
            architecture,
            all_users,
            property_table,
            directory_table,
            creating_application: summary_info.creating_application().map(str::to_owned),
            comments: summary_info.comments().map(str::to_owned),
        })
    }

    #[inline]
    fn build_directory(&self, current_dir: &str, target_dir: &str) -> Option<Utf8PathBuf> {
        self.directory_table
            .build_directory(current_dir, target_dir)
    }

    pub fn find_install_directory(&self) -> Option<Utf8PathBuf> {
        self.build_directory(INSTALL_DIR, TARGET_DIR)
            .or_else(|| {
                // Check the value of the `WIXUI_INSTALLDIR` property
                const WIX_UI_INSTALL_DIR: &str = "WIXUI_INSTALLDIR";

                self.property_table
                    .get(WIX_UI_INSTALL_DIR)
                    .and_then(|wix_install_dir| self.build_directory(wix_install_dir, TARGET_DIR))
            })
            .or_else(|| {
                // Check for an `INSTALLLOCATION` directory entry
                const INSTALL_LOCATION: &str = "INSTALLLOCATION";

                self.build_directory(INSTALL_LOCATION, TARGET_DIR)
            })
            .or_else(|| {
                // Check for an `APPDIR` directory entry
                const APP_DIR: &str = "APPDIR";

                self.build_directory(APP_DIR, TARGET_DIR)
            })
            .or_else(|| {
                // Find a directory entry with `installdir` in its name
                self.directory_table
                    .keys()
                    .find(|name| name.contains_ignore_ascii_case(INSTALL_DIR))
                    .and_then(|install_dir| self.build_directory(install_dir, TARGET_DIR))
            })
            .or_else(|| {
                // Get the first directory with zero or multiple subdirectories
                const SKIP_DIRECTORIES: [&str; 2] = ["DesktopFolder", "ProgramMenuFolder"];

                let mut path = Utf8PathBuf::new();
                let mut current_dir = TARGET_DIR;
                loop {
                    let sub_directories = self
                        .directory_table
                        .iter()
                        .filter(|(directory, (directory_parent, _))| {
                            !SKIP_DIRECTORIES.contains(&directory.as_str())
                                && directory_parent.as_deref() == Some(current_dir)
                        })
                        .collect::<Vec<_>>();

                    if sub_directories.len() == 1 {
                        let (directory, (_directory_parent, default_dir)) = sub_directories[0];
                        current_dir = directory;
                        path.push(
                            directory_table::get_property_relative_path(current_dir)
                                .unwrap_or(default_dir),
                        );
                    } else {
                        break;
                    }
                }
                Option::from(path).filter(|path| !path.as_str().is_empty())
            })
    }

    /// Returns the actual `ProductVersion` for Google Chrome.
    ///
    /// Google Chrome translates its `ProductVersion` into a different one. The actual
    /// `DisplayVersion` is retrieved from the MSI Summary Info Comments.
    ///
    /// See <https://issues.chromium.org/issues/382215764#comment8>.
    fn get_actual_chrome_version(&self) -> Option<&str> {
        self.comments
            .as_deref()
            .map(str::split_ascii_whitespace)
            .as_mut()
            .and_then(SplitAsciiWhitespace::next)
            .filter(|version| version.split('.').all(|part| part.parse::<u16>().is_ok()))
    }

    fn product_code(&self) -> Option<&str> {
        const PRODUCT_CODE: &str = "ProductCode";

        self.property_table.get(PRODUCT_CODE)
    }

    fn upgrade_code(&self) -> Option<&str> {
        const UPGRADE_CODE: &str = "UpgradeCode";

        self.property_table.get(UPGRADE_CODE)
    }

    fn product_name(&self) -> Option<&str> {
        const PRODUCT_NAME: &str = "ProductName";

        self.property_table.get(PRODUCT_NAME)
    }

    fn product_version(&self) -> Option<&str> {
        const GOOGLE_CHROME: &str = "Google Chrome";
        const PRODUCT_VERSION: &str = "ProductVersion";

        if self.product_name() == Some(GOOGLE_CHROME) {
            self.get_actual_chrome_version()
        } else {
            self.property_table.get(PRODUCT_VERSION)
        }
    }

    fn manufacturer(&self) -> Option<&str> {
        const MANUFACTURER: &str = "Manufacturer";

        self.property_table.get(MANUFACTURER)
    }

    fn product_language(&self) -> Option<LanguageTag> {
        const PRODUCT_LANGUAGE: &str = "ProductLanguage";

        let product_language = self.property_table.get(PRODUCT_LANGUAGE)?;
        let language_code = product_language.parse::<u16>().ok()?;

        Language::from_code(language_code)
            .tag()
            .parse::<LanguageTag>()
            .ok()
    }

    fn is_wix(&self) -> bool {
        const WIX: &str = "Wix";
        const WINDOWS_INSTALLER_XML: &str = "Windows Installer XML";

        // Check if the MSI has been created by WiX
        if self.creating_application.as_deref().is_some_and(|app| {
            app.contains_ignore_ascii_case(WIX)
                || app.contains_ignore_ascii_case(WINDOWS_INSTALLER_XML)
        }) {
            return true;
        }

        self.property_table.iter().any(|(property, value)| {
            property.contains_ignore_ascii_case(WIX) || value.contains_ignore_ascii_case(WIX)
        })
    }
}

impl Installers for Msi {
    fn installers(&self) -> Vec<Installer> {
        let product_code = self.product_code();
        let product_name = self.product_name();
        let manufacturer = self.manufacturer();
        let product_version = self.product_version();
        let upgrade_code = self.upgrade_code();

        let installer = Installer {
            locale: self.product_language(),
            architecture: self.architecture,
            r#type: Some(if self.is_wix() {
                InstallerType::Wix
            } else {
                InstallerType::Msi
            }),
            scope: self.all_users,
            product_code: product_code.map(str::to_owned),
            apps_and_features_entries: if product_name.is_some()
                || manufacturer.is_some()
                || product_version.is_some()
                || upgrade_code.is_some()
            {
                AppsAndFeaturesEntry::builder()
                    .maybe_display_name(product_name)
                    .maybe_publisher(manufacturer)
                    .maybe_display_version(product_version)
                    .maybe_product_code(product_code)
                    .maybe_upgrade_code(upgrade_code)
                    .build()
                    .into()
            } else {
                AppsAndFeaturesEntries::new()
            },
            installation_metadata: InstallationMetadata {
                default_install_location: self.find_install_directory(),
                ..InstallationMetadata::default()
            },
            ..Installer::default()
        };

        vec![installer]
    }
}
