use std::borrow::Cow;

use serde::Deserialize;

use super::bool_from_yes_no;

/// <https://github.com/wixtoolset/wix/blob/main/src/wix/WixToolset.Core.Burn/Bundles/CreateBurnManifestCommand.cs#L133>
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Variable {
    #[serde(rename = "@Id")]
    id: String,
    #[serde(rename = "@Value")]
    value: Option<String>,
    #[serde(rename = "@Type", default)]
    pub r#type: VariableType,
    #[serde(rename = "@Hidden", deserialize_with = "bool_from_yes_no", default)]
    pub hidden: bool,
    #[serde(rename = "@Persisted", deserialize_with = "bool_from_yes_no", default)]
    pub persisted: bool,
}

impl Variable {
    pub fn resolved_value(&self) -> Option<Cow<'_, str>> {
        /// <https://docs.firegiant.com/wix/tools/burn/builtin-variables/>
        const BUILT_IN_VARIABLES: [(&str, &str); 11] = [
            ("[AppDataFolder]", "%AppData%\\"),
            ("[CommonAppDataFolder]", "%ProgramData%\\"),
            ("[CommonFilesFolder]", "%CommonProgramFiles(x86)%\\"),
            ("[CommonFiles64Folder]", "%CommonProgramFiles%\\"),
            ("[CommonFiles6432Folder]", "%CommonProgramFiles%\\"),
            ("[LocalAppDataFolder", "%LocalAppData%\\"),
            ("[ProgramFilesFolder]", "%ProgramFiles(x86)%\\"),
            ("[ProgramFiles64Folder]", "%ProgramFiles%\\"),
            ("[ProgramFiles6432Folder]", "%ProgramFiles%\\"),
            ("[TempFolder]", "%Temp%\\"),
            ("[WindowsFolder]", "%WinDir%\\"),
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

    #[inline]
    pub const fn id(&self) -> &str {
        self.id.as_str()
    }

    #[expect(unused)]
    #[inline]
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
}

/// <https://github.com/wixtoolset/wix/blob/main/src/api/wix/WixToolset.Data/Symbols/WixBundleVariableSymbol.cs#L40>
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VariableType {
    #[default]
    Unknown,
    Formatted,
    Numeric,
    String,
    Version,
}
