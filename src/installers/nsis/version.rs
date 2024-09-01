use derive_more::Display;

#[derive(Debug, Display, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
#[display("{_0}.{_1}{_2}")]
pub struct NsisVersion(pub u8, pub u8, pub u8);

impl NsisVersion {
    pub fn from_branding_text(branding_text: &str) -> Option<Self> {
        let (_text, version) = branding_text.rsplit_once(' ')?;

        let mut parts = version
            .chars()
            .filter_map(|char| u8::try_from(char.to_digit(10)?).ok());

        Some(Self(parts.next()?, parts.next()?, parts.next()?))
    }

    pub const fn is_v3(self) -> bool {
        self.0 >= 3
    }
}

impl Default for NsisVersion {
    fn default() -> Self {
        Self(3, 0, 0)
    }
}
