mod language;
mod payload_ref;

use language::Language;
use payload_ref::PayloadRef;
use serde::Deserialize;

use super::super::super::bool_from_yes_no;

/// <https://github.com/wixtoolset/wix/blob/main/src/wix/WixToolset.Core.Burn/Bundles/CreateBurnManifestCommand.cs#L623>
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RelatedPackage<'manifest> {
    #[serde(rename = "@Id")]
    pub id: &'manifest str,
    #[serde(rename = "@MinVersion")]
    pub min_version: Option<&'manifest str>,
    #[serde(
        rename = "@MinInclusive",
        deserialize_with = "bool_from_yes_no",
        default
    )]
    pub min_inclusive: bool,
    #[serde(rename = "@MaxVersion")]
    pub max_version: Option<&'manifest str>,
    #[serde(
        rename = "@MaxInclusive",
        deserialize_with = "bool_from_yes_no",
        default
    )]
    pub max_inclusive: bool,
    #[serde(rename = "@OnlyDetect", deserialize_with = "bool_from_yes_no", default)]
    pub only_detect: bool,
    #[serde(
        rename = "@LangInclusive",
        deserialize_with = "bool_from_yes_no",
        default
    )]
    pub lang_inclusive: bool,
    #[serde(rename = "Language", default, borrow)]
    pub languages: Vec<Language<'manifest>>,
    #[serde(rename = "PayloadRef", default, borrow)]
    pub payload_refs: Vec<PayloadRef<'manifest>>,
}
