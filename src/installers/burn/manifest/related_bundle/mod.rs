mod action;

use action::Action;
use serde::Deserialize;

/// <https://docs.firegiant.com/wix/schema/wxs/relatedbundle/>
#[expect(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RelatedBundle<'manifest> {
    #[serde(rename = "@Code", alias = "@Id")]
    pub code: &'manifest str,
    #[serde(rename = "@Action")]
    pub action: Action,
}
