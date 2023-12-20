use nutype::nutype;

#[nutype(
    validate(len_char_min = 1, len_char_max = 10000),
    derive(Clone, FromStr, Display, Deserialize, Serialize)
)]
pub struct InstallationNotes(String);
