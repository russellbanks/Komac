use std::io;

use zerocopy::FromBytes;

use super::{VSHeader, VarDword};

#[derive(Clone, Debug)]
pub struct VSVar<'a> {
    header: VSHeader<'a>,
    children: &'a [VarDword],
}

impl<'a> VSVar<'a> {
    pub fn read_from(data: &'a [u8]) -> io::Result<Self> {
        let header = VSHeader::read_from(data)?;

        let children =
            <[VarDword]>::ref_from_bytes(&data[header.end_offset..usize::from(header.length())])
                .map_err(|err| io::Error::other(err.to_string()))?;

        Ok(Self { header, children })
    }

    #[must_use]
    #[inline]
    pub const fn length(&self) -> u16 {
        self.header.length()
    }

    #[must_use]
    #[inline]
    pub const fn value_length(&self) -> u16 {
        self.header.value_length()
    }
}
