use std::{
    fmt,
    io::{Read, Seek, SeekFrom},
};

use crate::analysis::installers::{
    nsis::{FirstHeader, NsisError},
    pe::PE,
};

pub struct FirstHeaderFinder {
    offset: u64,
    first_header: FirstHeader,
}

impl FirstHeaderFinder {
    /// The maximum offset to check for a [`FirstHeader`] at.
    const MAX_SEARCH: u64 = 1 << 20; // 1 MiB

    /// The step size that the searcher will move forward by each iteration.
    const STEP: u64 = 512;

    /// Searches for a [FirstHeader].
    ///
    /// If a [FirstHeader] is found, the reader's position will be at the end of
    /// the [FirstHeader].
    /// If one is not found, the reader's position will be be 1 MiB + 512 B from
    /// the start of the file.
    pub fn find<R: Read + Seek>(mut reader: R, pe: &PE) -> Result<Self, NsisError> {
        // Start at the PE overlay offset
        let mut offset = pe
            .overlay_offset()
            .ok_or(NsisError::NotNsisFile)?
            .next_multiple_of(Self::STEP);

        while offset <= Self::MAX_SEARCH {
            // Seek to the offset
            reader
                .seek(SeekFrom::Start(offset))
                .map_err(|_| NsisError::NotNsisFile)?;

            // Try read a FirstHeader at the current position
            if let Ok(first_header) = FirstHeader::try_read_from_io(&mut reader) {
                return Ok(Self {
                    offset,
                    first_header,
                });
            }

            // Step forward by 512 bytes
            offset += Self::STEP;
        }

        Err(NsisError::NotNsisFile)
    }

    /// Returns the offset to the [FirstHeader].
    #[expect(unused)]
    #[must_use]
    #[inline]
    pub const fn offset(&self) -> u64 {
        self.offset
    }

    /// Returns a reference to the [FirstHeader].
    #[must_use]
    #[inline]
    pub const fn first_header(&self) -> &FirstHeader {
        &self.first_header
    }

    /// Returns the offset to the data.
    #[must_use]
    #[inline]
    pub const fn data_offset(&self) -> u64 {
        self.offset + size_of::<FirstHeader>() as u64
    }
}

impl fmt::Debug for FirstHeaderFinder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FirstHeaderFinder")
            .field("offset", &self.offset)
            .field("first_header", &self.first_header)
            .field("data_offset", &self.data_offset())
            .finish()
    }
}
