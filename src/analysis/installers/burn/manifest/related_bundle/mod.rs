mod action;

use action::Action;
use serde::Deserialize;
use uuid::{Uuid, fmt::Braced};

/// <https://docs.firegiant.com/wix/schema/wxs/relatedbundle/>
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct RelatedBundle {
    /// The identifier of the `RelatedBundle` group.
    #[serde(rename = "@Code", alias = "@Id")]
    code: Uuid,

    /// The action to take on bundles related to this one.
    #[serde(rename = "@Action")]
    action: Action,
}

impl RelatedBundle {
    #[inline]
    pub fn code(&self) -> &Braced {
        self.code.as_braced()
    }

    #[expect(unused)]
    #[inline]
    pub const fn action(&self) -> Action {
        self.action
    }
}
