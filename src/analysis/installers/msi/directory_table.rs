use std::{
    io,
    io::{Read, Seek},
};

use camino::Utf8PathBuf;
use indexmap::{
    IndexMap,
    map::{Iter, Keys},
};
use msi::{Package, Select};

use crate::analysis::installers::utils::{
    RELATIVE_APP_DATA, RELATIVE_COMMON_FILES_32, RELATIVE_COMMON_FILES_64, RELATIVE_LOCAL_APP_DATA,
    RELATIVE_PROGRAM_FILES_32, RELATIVE_PROGRAM_FILES_64, RELATIVE_TEMP_FOLDER,
    RELATIVE_WINDOWS_DIR,
};

/// Represents the [Windows Installer Directory Table](https://learn.microsoft.com/windows/win32/msi/directory-table),
/// which defines the logical and physical directory layout of an MSI package.
///
/// Each entry in the Directory Table describes both a **source** and **target** directory.
/// The table forms a hierarchy using parent–child relationships defined by the `Directory`
/// and `Directory_Parent` columns.
///
/// The underlying data is stored as an ordered map of:
///
/// ```text
/// Directory identifier → (Optional parent directory identifier, default directory name)
/// ```
///
/// # Structure
///
/// | Column            | Description |
/// |-------------------|--------------|
/// | **`Directory`**     | Unique identifier (or property name) for this directory. |
/// | **`Directory_Parent`** | Identifier of the parent directory, or `NULL` if this is the root. |
/// | **`DefaultDir`**    | The directory name (may be `target:source` or `short|long` form). |
///
/// # Example
///
/// ```text
/// Directory          Directory_Parent   DefaultDir
/// ---------------------------------------------------------
/// TARGETDIR          TARGETDIR          SourceDir
/// ProgramFilesFolder TARGETDIR          .:ProgramFiles
/// MyAppDir           ProgramFilesFolder MyApp
/// ```
///
/// In this example:
/// - `TARGETDIR` is the root.
/// - `ProgramFilesFolder` is a subdirectory of `TARGETDIR`.
/// - `MyAppDir` is a subdirectory of `ProgramFilesFolder`.
///
/// # Notes
///
/// - Only one root directory (`TARGETDIR`) may exist.
/// - Directory resolution occurs during the MSI `CostFinalize` action.
/// - The `DefaultDir` value defines the subdirectory name under the parent directory.
/// - Certain MSI properties (e.g., `ProgramFilesFolder`, `AppDataFolder`) map to system locations.
///
/// This type provides helper functions to construct a directory hierarchy and to
/// resolve MSI directory identifiers into concrete paths.
///
/// See also:
/// - [Using the Directory Table](https://learn.microsoft.com/windows/win32/msi/using-the-directory-table)
/// - [Property Reference](https://learn.microsoft.com/windows/win32/msi/property-reference)
#[derive(Clone, Debug)]
pub struct DirectoryTable(IndexMap<String, (Option<String>, String)>);

impl DirectoryTable {
    pub fn new<R: Read + Seek>(msi: &mut Package<R>) -> io::Result<Self> {
        const DIRECTORY: &str = "Directory";
        const DIRECTORY_PARENT: &str = "Directory_Parent";
        const DEFAULT_DIR: &str = "DefaultDir";

        Ok(Self(
            msi.select_rows(Select::table(DIRECTORY))?
                .filter_map(|row| {
                    match (
                        row[DIRECTORY].as_str(),
                        row[DIRECTORY_PARENT].as_str(),
                        row[DEFAULT_DIR].as_str().map(|default_dir| {
                            default_dir
                                .split_once('|')
                                .map_or(default_dir, |(_, long_dir)| long_dir)
                        }),
                    ) {
                        (Some(directory), parent, Some(default)) => Some((
                            directory.to_owned(),
                            (parent.map(str::to_owned), default.to_owned()),
                        )),
                        _ => None,
                    }
                })
                .collect::<IndexMap<String, (Option<String>, String)>>(),
        ))
    }

    /// Constructs a path from the root directory to the target subdirectory based on the directory
    /// table.
    ///
    /// This is deliberately recursive so that the function can start at the deepest directory,
    /// traverse upwards, and then build the path sequentially as the stack is unwinding. Using a
    /// loop would require the path components to be reversed at the end.
    ///
    /// [Using the Directory Table](https://learn.microsoft.com/windows/win32/msi/using-the-directory-table)
    pub fn build_directory(&self, current_dir: &str, target_dir: &str) -> Option<Utf8PathBuf> {
        // If the current directory is the target, return an empty path
        if current_dir == target_dir {
            return Some(Utf8PathBuf::new());
        }

        if let Some((Some(parent), default_dir)) = self.0.get(current_dir)
            && let Some(mut path) = self.build_directory(parent, target_dir)
        {
            path.push(get_property_relative_path(current_dir).unwrap_or(default_dir));
            return Some(path);
        }

        None
    }

    /// Returns an iterator over all directory identifiers (the `Directory` column) in the order
    /// they appear in the MSI table.
    /// #[inline]
    pub fn keys(&self) -> Keys<'_, String, (Option<String>, String)> {
        self.0.keys()
    }

    /// Returns an iterator over all `(Directory, (Directory_Parent, DefaultDir))` pairs in their
    /// original order.
    #[inline]
    pub fn iter(&self) -> Iter<'_, String, (Option<String>, String)> {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a DirectoryTable {
    type Item = (&'a String, &'a (Option<String>, String));

    type IntoIter = Iter<'a, String, (Option<String>, String)>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

pub fn get_property_relative_path(property: &str) -> Option<&str> {
    const PROGRAM_FILES_64_FOLDER: &str = "ProgramFiles64Folder";
    const PROGRAM_FILES_FOLDER: &str = "ProgramFilesFolder";
    const COMMON_FILES_64_FOLDER: &str = "CommonFiles64Folder";
    const COMMON_FILES_FOLDER: &str = "CommonFilesFolder";
    const APP_DATA_FOLDER: &str = "AppDataFolder";
    const LOCAL_APP_DATA_FOLDER: &str = "LocalAppDataFolder";
    const TEMP_FOLDER: &str = "TempFolder";
    const WINDOWS_FOLDER: &str = "WindowsFolder";

    match property {
        PROGRAM_FILES_64_FOLDER => Some(RELATIVE_PROGRAM_FILES_64),
        PROGRAM_FILES_FOLDER => Some(RELATIVE_PROGRAM_FILES_32),
        COMMON_FILES_64_FOLDER => Some(RELATIVE_COMMON_FILES_64),
        COMMON_FILES_FOLDER => Some(RELATIVE_COMMON_FILES_32),
        APP_DATA_FOLDER => Some(RELATIVE_APP_DATA),
        LOCAL_APP_DATA_FOLDER => Some(RELATIVE_LOCAL_APP_DATA),
        TEMP_FOLDER => Some(RELATIVE_TEMP_FOLDER),
        WINDOWS_FOLDER => Some(RELATIVE_WINDOWS_DIR),
        _ => None,
    }
}
