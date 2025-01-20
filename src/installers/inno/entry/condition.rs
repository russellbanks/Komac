use crate::installers::inno::encoding::InnoValue;
use crate::installers::inno::version::KnownVersion;
use encoding_rs::Encoding;
use std::io::{Read, Result};

#[derive(Debug, Default)]
pub struct Condition {
    components: Option<String>,
    tasks: Option<String>,
    languages: Option<String>,
    check: Option<String>,
    after_install: Option<String>,
    before_install: Option<String>,
}

impl Condition {
    pub fn load<R: Read>(
        reader: &mut R,
        codepage: &'static Encoding,
        version: &KnownVersion,
    ) -> Result<Self> {
        let mut condition = Self::default();

        if *version >= (2, 0, 0) || (version.is_isx() && *version >= (1, 3, 8)) {
            condition.components = InnoValue::new_string(reader, codepage)?;
        }

        if *version >= (2, 0, 0) || (version.is_isx() && *version >= (1, 3, 17)) {
            condition.tasks = InnoValue::new_string(reader, codepage)?;
        }

        if *version >= (4, 0, 1) {
            condition.languages = InnoValue::new_string(reader, codepage)?;
        }

        if *version >= (4, 0, 0) || (version.is_isx() && *version >= (1, 3, 24)) {
            condition.check = InnoValue::new_string(reader, codepage)?;
        }

        if *version >= (4, 1, 0) {
            condition.after_install = InnoValue::new_string(reader, codepage)?;
            condition.before_install = InnoValue::new_string(reader, codepage)?;
        }

        Ok(condition)
    }
}
