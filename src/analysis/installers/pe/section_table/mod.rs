mod section_header;

use std::{io, slice, vec};

pub use section_header::SectionHeader;
use zerocopy::{FromZeros, IntoBytes};

use super::CoffHeader;

#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct SectionTable(Vec<SectionHeader>);

impl SectionTable {
    #[inline]
    pub fn sections(&self) -> &[SectionHeader] {
        &self.0
    }

    pub fn read_from<R>(mut src: R, coff_header: CoffHeader) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut sections =
            vec![SectionHeader::new_zeroed(); coff_header.number_of_sections().into()];

        for section in &mut sections {
            src.read_exact(section.as_mut_bytes())?;
        }

        Ok(Self(sections))
    }

    /// Converts a virtual address (RVA) to a file offset.
    ///
    /// # Errors
    ///
    /// Returns an error if the address was not found in any section.
    pub fn to_file_offset(&self, address: u32) -> io::Result<u32> {
        for section in self {
            let start = section.virtual_address();
            let end = start.saturating_add(section.virtual_size());

            if (start..end).contains(&address) {
                return Ok(address + section.pointer_to_raw_data() - start);
            }
        }

        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Address {address:#X} not found in any section"),
        ))
    }
}

impl<'table> IntoIterator for &'table SectionTable {
    type Item = &'table SectionHeader;

    type IntoIter = slice::Iter<'table, SectionHeader>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
