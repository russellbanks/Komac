use serde::Deserialize;

/// The head commit's relation to the base commit in a comparison.
#[derive(Copy, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Diverged,
    Ahead,
    Behind,
    Identical,
}
