use std::cmp::Ordering;

use derive_more::{Deref, Display, FromStr};
use icu_locid::{LanguageIdentifier, langid};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Display, Deref, Eq, PartialEq, FromStr, Hash, Deserialize, Serialize)]
pub struct LanguageTag(LanguageIdentifier);

impl Default for LanguageTag {
    fn default() -> Self {
        Self(langid!("en-US"))
    }
}

impl PartialOrd for LanguageTag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LanguageTag {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.total_cmp(&self.0)
    }
}
