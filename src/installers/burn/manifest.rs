use crate::installers::utils::{
    RELATIVE_APP_DATA, RELATIVE_COMMON_FILES_32, RELATIVE_COMMON_FILES_64, RELATIVE_LOCAL_APP_DATA,
    RELATIVE_PROGRAM_DATA, RELATIVE_PROGRAM_FILES_32, RELATIVE_PROGRAM_FILES_64,
    RELATIVE_TEMP_FOLDER, RELATIVE_WINDOWS_DIR,
};
use crate::types::version::Version;
use const_format::concatcp;
use derive_more::Deref;
use serde::de::{Unexpected, Visitor};
use serde::{Deserialize, Deserializer};
use std::borrow::Cow;
use std::fmt::Formatter;

#[expect(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BurnManifest<'manifest> {
    #[serde(rename = "@EngineVersion")]
    pub engine_version: Option<Version>,
    #[serde(rename = "@ProtocolVersion")]
    pub protocol_version: Option<u8>,
    #[serde(rename = "@Win64", default)]
    pub win_64: YesNo,
    #[serde(borrow)]
    pub related_bundle: RelatedBundle<'manifest>,
    #[serde(rename = "Variable", borrow)]
    pub variables: Vec<Variable<'manifest>>,
    #[serde(rename = "Payload", borrow)]
    pub payloads: Vec<Payload<'manifest>>,
    #[serde(borrow)]
    pub registration: Registration<'manifest>,
    #[serde(borrow)]
    pub chain: Chain<'manifest>,
}

#[expect(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RelatedBundle<'manifest> {
    #[serde(rename = "@Code", alias = "@Id")]
    pub code: &'manifest str,
    #[serde(rename = "@Action")]
    pub action: Action,
}

/// <https://github.com/wixtoolset/wix/blob/main/src/api/wix/WixToolset.Data/Symbols/WixRelatedBundleSymbol.cs#L32>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Action {
    Detect,
    Upgrade,
    Addon,
    Patch,
}

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
    #[serde(rename = "@Hidden", default)]
    pub hidden: YesNo,
    #[serde(rename = "@Persisted", default)]
    pub persisted: YesNo,
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
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VariableType {
    #[default]
    Unknown,
    Formatted,
    Numeric,
    String,
    Version,
}

/// <https://github.com/wixtoolset/wix/blob/main/src/wix/WixToolset.Core.Burn/Bundles/CreateBurnManifestCommand.cs#L753>
#[expect(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Payload<'manifest> {
    #[serde(rename = "@Id")]
    pub id: &'manifest str,
    #[serde(rename = "@FilePath")]
    pub file_path: &'manifest str,
    #[serde(rename = "@FileSize")]
    pub file_size: &'manifest str,
    #[serde(rename = "@CertificateRootPublicKeyIdentifier")]
    pub certificate_root_public_key_identifier: Option<&'manifest str>,
    #[serde(rename = "@CertificateRootThumbprint")]
    pub certificate_root_thumbprint: Option<&'manifest str>,
    #[serde(rename = "@Hash")]
    pub hash: Option<&'manifest str>,
    #[serde(rename = "@LayoutOnly", default)]
    pub layout_only: YesNo,
    #[serde(rename = "@DownloadUrl")]
    pub download_url: Option<&'manifest str>,
    #[serde(rename = "@Packaging", default)]
    pub packaging: Packaging,
    #[serde(rename = "@SourcePath")]
    pub source_path: &'manifest str,
    #[serde(rename = "@Container")]
    pub container: Option<&'manifest str>,
}

/// <https://github.com/wixtoolset/wix/blob/main/src/api/wix/WixToolset.Data/PackagingType.cs#L5>
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Packaging {
    #[default]
    Unknown,
    Embedded,
    External,
}

#[expect(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Registration<'manifest> {
    #[serde(rename = "@Code", alias = "@Id")]
    pub id: &'manifest str,
    #[serde(rename = "@ExecutableName")]
    pub executable_name: &'manifest str,
    #[serde(rename = "@PerMachine")]
    pub per_machine: YesNo,
    #[serde(rename = "@Tag")]
    pub tag: &'manifest str,
    #[serde(rename = "@Version")]
    pub version: &'manifest str,
    #[serde(rename = "@ProviderKey")]
    pub provider_key: &'manifest str,
    pub arp: Arp<'manifest>,
}

#[expect(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Arp<'manifest> {
    #[serde(rename = "@Register", default)]
    pub register: YesNo,
    #[serde(rename = "@DisplayName")]
    pub display_name: &'manifest str,
    #[serde(rename = "@DisplayVersion")]
    pub display_version: Version,
    #[serde(rename = "@InProgressDisplayName")]
    pub in_progress_display_name: Option<&'manifest str>,
    #[serde(rename = "@Publisher")]
    pub publisher: Option<&'manifest str>,
    #[serde(rename = "@HelpLink")]
    pub help_link: Option<&'manifest str>,
    #[serde(rename = "@HelpTelephone")]
    pub help_telephone: Option<&'manifest str>,
    #[serde(rename = "@AboutUrl")]
    pub about_url: Option<&'manifest str>,
    #[serde(rename = "@UpdateUrl")]
    pub update_url: Option<&'manifest str>,
    #[serde(rename = "@ParentDisplayName")]
    pub parent_display_name: Option<&'manifest str>,
    #[serde(rename = "@DisableModify", default)]
    pub disable_modify: YesNoButton,
    #[serde(rename = "@DisableRemove", default)]
    pub disable_remove: YesNo,
}

#[expect(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Chain<'manifest> {
    #[serde(rename = "@DisableRollback", default)]
    pub disable_rollback: YesNo,
    #[serde(rename = "@DisableSystemRestore", default)]
    pub disable_system_restore: YesNo,
    #[serde(rename = "@ParallelCache", default)]
    pub parallel_cache: YesNo,
    #[serde(rename = "$value", borrow)]
    pub packages: Vec<Package<'manifest>>,
}

#[expect(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Package<'manifest> {
    #[serde(rename = "BundlePackage")]
    Bundle,
    #[serde(rename = "ExePackage", borrow)]
    Exe(ExePackage<'manifest>),
    #[serde(rename = "MsiPackage", borrow)]
    Msi(MsiPackage<'manifest>),
    #[serde(rename = "MspPackage")]
    Msp,
    #[serde(rename = "MsuPackage")]
    Msu,
}

#[expect(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExePackage<'manifest> {
    #[serde(rename = "@Id")]
    pub id: &'manifest str,
}

#[expect(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MsiPackage<'manifest> {
    #[serde(rename = "@Id")]
    pub id: &'manifest str,
    #[serde(rename = "@ProductCode")]
    pub product_code: &'manifest str,
    #[serde(rename = "@Language")]
    pub language: &'manifest str,
    #[serde(rename = "@Version")]
    pub version: Version,
    #[serde(rename = "@UpgradeCode")]
    pub upgrade_code: Option<&'manifest str>,
    #[serde(rename = "MsiProperty")]
    pub properties: Vec<MsiProperty<'manifest>>,
    pub provides: Provides<'manifest>,
}

#[expect(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MsiProperty<'manifest> {
    #[serde(rename = "@Id")]
    pub id: &'manifest str,
    #[serde(rename = "@Value")]
    pub value: &'manifest str,
    #[serde(rename = "@Condition")]
    pub condition: Option<&'manifest str>,
}

impl MsiProperty<'_> {
    pub fn is_arp_system_component(&self) -> bool {
        const ARP_SYSTEM_COMPONENT: &str = "ARPSYSTEMCOMPONENT";

        self.id == ARP_SYSTEM_COMPONENT
    }
}

#[expect(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Provides<'manifest> {
    #[serde(rename = "@Key")]
    pub key: &'manifest str,
    #[serde(rename = "@Version")]
    pub version: Option<&'manifest str>,
    #[serde(rename = "@DisplayName")]
    pub display_name: Option<&'manifest str>,
    #[serde(rename = "@Imported", default)]
    pub imported: YesNoButton,
}

#[expect(dead_code)]
#[derive(derive_more::Debug)]
pub enum YesNoButton {
    #[debug("{_0}")]
    YesNo(bool),
    Button,
}

impl Default for YesNoButton {
    fn default() -> Self {
        Self::YesNo(false)
    }
}

impl<'de> Deserialize<'de> for YesNoButton {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct YesNoButtonVisitor;

        impl Visitor<'_> for YesNoButtonVisitor {
            type Value = YesNoButton;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("`yes`/`true`, `no`/`false`, or `button`")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "yes" | "true" => Ok(YesNoButton::YesNo(true)),
                    "no" | "false" => Ok(YesNoButton::YesNo(false)),
                    "button" => Ok(YesNoButton::Button),
                    _ => Err(E::invalid_value(Unexpected::Str(value), &self)),
                }
            }
        }

        deserializer.deserialize_string(YesNoButtonVisitor)
    }
}

#[derive(derive_more::Debug, Default, Deref)]
#[debug("{_0}")]
#[repr(transparent)]
pub struct YesNo(bool);

impl<'de> Deserialize<'de> for YesNo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct YesNoVisitor;

        impl Visitor<'_> for YesNoVisitor {
            type Value = YesNo;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("`yes`/`true` or `no`/`false`")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "yes" | "true" => Ok(YesNo(true)),
                    "no" | "false" => Ok(YesNo(false)),
                    _ => Err(E::invalid_value(Unexpected::Str(value), &self)),
                }
            }
        }

        deserializer.deserialize_string(YesNoVisitor)
    }
}
