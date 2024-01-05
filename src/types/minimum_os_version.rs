use nutype::nutype;
use std::str::FromStr;

#[nutype(
    validate(predicate = is_minimum_os_version_valid),
    default = "0.0.0.0",
    derive(Clone, FromStr, Debug, Default, Deref, Display, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)
)]
pub struct MinimumOSVersion(String);

fn is_minimum_os_version_valid(input: &str) -> bool {
    let parts = input.split('.').collect::<Vec<_>>();

    let parts_count = parts.len();
    if !(1..=4).contains(&parts_count) {
        return false;
    }

    if parts.into_iter().any(|part| u16::from_str(part).is_err()) {
        return false;
    }

    true
}
