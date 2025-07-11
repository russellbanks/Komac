use std::borrow::Cow;

use const_format::concatcp;
use serde::Deserialize;

use super::bool_from_yes_no;
use crate::installers::utils::{
    RELATIVE_APP_DATA, RELATIVE_COMMON_FILES_32, RELATIVE_COMMON_FILES_64, RELATIVE_LOCAL_APP_DATA,
    RELATIVE_PROGRAM_DATA, RELATIVE_PROGRAM_FILES_32, RELATIVE_PROGRAM_FILES_64,
    RELATIVE_TEMP_FOLDER, RELATIVE_WINDOWS_DIR,
};

/// <https://github.com/wixtoolset/wix/blob/main/src/wix/WixToolset.Core.Burn/Bundles/CreateBurnManifestCommand.cs#L133>
#[expect(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Variable<'manifest> {
    #[serde(rename = "@Id")]
    pub id: &'manifest str,
    #[serde(rename = "@Value")]
    pub value: Option<Cow<'manifest, str>>,
    #[serde(rename = "@Type", default)]
    pub r#type: VariableType,
    #[serde(rename = "@Hidden", deserialize_with = "bool_from_yes_no", default)]
    pub hidden: bool,
    #[serde(rename = "@Persisted", deserialize_with = "bool_from_yes_no", default)]
    pub persisted: bool,
}

impl<'manifest> Variable<'manifest> {
    pub fn resolved_value(&'manifest self) -> Option<Cow<'manifest, str>> {
        /// <https://docs.firegiant.com/wix/tools/burn/builtin-variables/>
        const BUILT_IN_VARIABLES: [(&str, &str); 11] = [
            ("[AppDataFolder]", concatcp!(RELATIVE_APP_DATA, '\\')),
            (
                "[CommonAppDataFolder]",
                concatcp!(RELATIVE_PROGRAM_DATA, '\\'),
            ),
            (
                "[CommonFilesFolder]",
                concatcp!(RELATIVE_COMMON_FILES_32, '\\'),
            ),
            (
                "[CommonFiles64Folder]",
                concatcp!(RELATIVE_COMMON_FILES_64, '\\'),
            ),
            (
                "[CommonFiles6432Folder]",
                concatcp!(RELATIVE_COMMON_FILES_64, '\\'),
            ),
            (
                "[LocalAppDataFolder",
                concatcp!(RELATIVE_LOCAL_APP_DATA, '\\'),
            ),
            (
                "[ProgramFilesFolder]",
                concatcp!(RELATIVE_PROGRAM_FILES_32, '\\'),
            ),
            (
                "[ProgramFiles64Folder]",
                concatcp!(RELATIVE_PROGRAM_FILES_64, '\\'),
            ),
            (
                "[ProgramFiles6432Folder]",
                concatcp!(RELATIVE_PROGRAM_FILES_64, '\\'),
            ),
            ("[TempFolder]", concatcp!(RELATIVE_TEMP_FOLDER, '\\')),
            ("[WindowsFolder]", concatcp!(RELATIVE_WINDOWS_DIR, '\\')),
        ];

        const ESCAPES: [(&str, &str); 1] = [("&quot;", r#"""#)];

        let mut value = self.value.as_deref().map(Cow::Borrowed)?;

        for (escape, replacement) in ESCAPES {
            while let Some(index) = value.find(escape) {
                value
                    .to_mut()
                    .replace_range(index..index + escape.len(), replacement);
            }
        }

        for (variable, replacement) in BUILT_IN_VARIABLES {
            if let Some(index) = value.find(variable) {
                value
                    .to_mut()
                    .replace_range(index..index + variable.len(), replacement);
                break;
            }
        }

        Some(value)
    }
}

/// <https://github.com/wixtoolset/wix/blob/main/src/api/wix/WixToolset.Data/Symbols/WixBundleVariableSymbol.cs#L40>
#[derive(Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VariableType {
    #[default]
    Unknown,
    Formatted,
    Numeric,
    String,
    Version,
}
