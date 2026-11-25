use serde::Deserialize;

/// The head commit's relation to the base commit in a comparison.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompareStatus {
    Diverged,
    Ahead,
    Behind,
    Identical,
}
