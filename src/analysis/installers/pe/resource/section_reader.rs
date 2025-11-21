use std::{
    cmp, fmt,
    io::{self, Read, Seek, SeekFrom},
};

pub struct SectionReader<R> {
    inner: R,
    start: u64,
    length: u64,
    position: u64, // Position within the section (0-based)
}

impl<R: Read + Seek> SectionReader<R> {
    pub fn new(mut inner: R, start: u64, length: u64) -> io::Result<Self> {
        // Seek to the start of the section
        inner.seek(SeekFrom::Start(start))?;

        Ok(Self {
            inner,
            start,
            length,
            position: 0,
        })
    }

    /// Returns the current position within the section (0-based).
    #[inline]
    pub const fn position(&self) -> u64 {
        self.position
    }

    /// Returns the remaining bytes in the section.
    #[inline]
    pub const fn remaining(&self) -> u64 {
        self.length.saturating_sub(self.position)
    }

    /// Returns the section start offset in the underlying reader.
    #[inline]
    pub const fn section_start(&self) -> u64 {
        self.start
    }

    /// Returns the section length.
    #[inline]
    pub const fn section_length(&self) -> u64 {
        self.length
    }
}

impl<R: Read + Seek> Read for SectionReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let remaining = self.remaining();
        if remaining == 0 {
            return Ok(0); // EOF
        }

        // Limit read to remaining bytes in section
        let to_read = cmp::min(buf.len() as u64, remaining) as usize;
        let bytes_read = self.inner.read(&mut buf[..to_read])?;

        self.position += bytes_read as u64;
        Ok(bytes_read)
    }
}

impl<R: Read + Seek> Seek for SectionReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_position = match pos {
            SeekFrom::Start(offset) => offset,
            SeekFrom::End(offset) => self.length.saturating_add_signed(offset),
            SeekFrom::Current(offset) => self.position.saturating_add_signed(offset),
        };

        // Clamp to section bounds
        let clamped_position = cmp::min(new_position, self.length);

        // Seek in the underlying reader to the absolute position
        let absolute_position = self.start + clamped_position;
        self.inner.seek(SeekFrom::Start(absolute_position))?;

        self.position = clamped_position;
        Ok(self.position)
    }
}

impl<R> fmt::Debug for SectionReader<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SectionReader")
            .field("start", &self.start)
            .field("length", &self.length)
            .field("position", &self.position)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read, Seek, SeekFrom};

    use super::SectionReader;

    fn create_test_data() -> Vec<u8> {
        (0..100u8).collect()
    }

    #[test]
    fn basic_section_reading() {
        let mut reader = SectionReader::new(Cursor::new(create_test_data()), 15, 20).unwrap();

        // Read partial data
        let mut buf = [0u8; 8];
        let bytes_read = reader.read(&mut buf).unwrap();
        assert_eq!(bytes_read, 8);
        assert_eq!(buf, [15, 16, 17, 18, 19, 20, 21, 22]);

        // Read remaining data (should be limited to section)
        let mut buf = [0u8; 20];
        let bytes_read = reader.read(&mut buf).unwrap();
        assert_eq!(bytes_read, 12); // Only 12 bytes left in section
        assert_eq!(
            &buf[..12],
            &[23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34]
        );

        // Should be EOF now
        assert_eq!(reader.read(&mut buf).unwrap(), 0);
    }

    #[test]
    fn negative_seek_operations() {
        let mut reader = SectionReader::new(Cursor::new(create_test_data()), 20, 30).unwrap();

        // Move to middle, then seek backwards with Current
        reader.seek(SeekFrom::Start(15)).unwrap();
        reader.seek(SeekFrom::Current(-5)).unwrap();
        assert_eq!(reader.position(), 10);

        // Seek to near end, then backwards with End
        reader.seek(SeekFrom::End(-3)).unwrap();
        assert_eq!(reader.position(), 27);

        // Test underflow protection - should clamp to 0
        reader.seek(SeekFrom::Current(-100)).unwrap();
        assert_eq!(reader.position(), 0);

        reader.seek(SeekFrom::End(-100)).unwrap();
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn chunked_streaming_read() {
        let mut reader = SectionReader::new(Cursor::new(create_test_data()), 25, 15).unwrap();

        let mut result = Vec::new();
        let mut buf = [0u8; 4]; // Small chunks

        while let Ok(bytes_read) = reader.read(&mut buf) {
            if bytes_read == 0 {
                break;
            }
            result.extend_from_slice(&buf[..bytes_read]);
        }

        let expected: Vec<u8> = (25..40).collect();
        assert_eq!(result, expected);
        assert_eq!(result.len(), 15);
    }

    #[test]
    fn random_access_seeking() {
        let mut reader = SectionReader::new(Cursor::new(create_test_data()), 10, 40).unwrap();

        // Jump to end
        reader.seek(SeekFrom::End(0)).unwrap();
        assert_eq!(reader.position(), 40);

        // Jump to middle
        reader.seek(SeekFrom::Start(20)).unwrap();
        let mut buf = [0u8; 1];
        reader.read(&mut buf).unwrap();
        assert_eq!(buf[0], 30); // 10 + 20

        // Relative jump backwards
        reader.seek(SeekFrom::Current(-10)).unwrap();
        reader.read(&mut buf).unwrap();
        assert_eq!(buf[0], 21); // 10 + 11 (position after previous read was 21)

        // Jump beyond end (should clamp)
        reader.seek(SeekFrom::Start(50)).unwrap();
        assert_eq!(reader.position(), 40);
    }

    #[test]
    fn edge_case_sections() {
        // Empty section
        let mut empty_reader = SectionReader::new(Cursor::new(create_test_data()), 50, 0).unwrap();
        let mut buf = [0u8; 10];
        assert_eq!(empty_reader.read(&mut buf).unwrap(), 0);
        assert_eq!(empty_reader.remaining(), 0);

        // Single byte section
        let mut single_reader = SectionReader::new(Cursor::new(create_test_data()), 42, 1).unwrap();
        assert_eq!(single_reader.read(&mut buf).unwrap(), 1);
        assert_eq!(buf[0], 42);
        assert_eq!(single_reader.read(&mut buf).unwrap(), 0); // EOF

        // Section at file boundaries
        let mut start_reader = SectionReader::new(Cursor::new(create_test_data()), 0, 5).unwrap();
        start_reader.read(&mut buf).unwrap();
        assert_eq!(&buf[..5], &[0, 1, 2, 3, 4]);

        let mut end_reader = SectionReader::new(Cursor::new(create_test_data()), 95, 5).unwrap();
        end_reader.read(&mut buf).unwrap();
        assert_eq!(&buf[..5], &[95, 96, 97, 98, 99]);
    }
}
