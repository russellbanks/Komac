use nutype::nutype;

#[nutype(
    validate(len_char_min = 1, len_char_max = 512),
    derive(
        Clone,
        FromStr,
        Debug,
        Display,
        Deserialize,
        Serialize,
        Eq,
        PartialEq,
        PartialOrd,
        Ord,
        Hash
    )
)]
pub struct InstallerSwitch(String);
