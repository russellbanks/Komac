use std::io::{Read, Result};

use encoding_rs::Encoding;

use crate::installers::inno::encoding::InnoValue;

#[expect(dead_code)]
#[derive(Debug, Default)]
pub struct Permission(String);

impl Permission {
    pub fn from_reader<R: Read>(reader: &mut R, codepage: &'static Encoding) -> Result<Self> {
        InnoValue::new_string(reader, codepage)
            .map(Option::unwrap_or_default)
            .map(Permission)
    }
}
