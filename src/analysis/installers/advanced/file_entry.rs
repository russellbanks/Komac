use std::{
    fmt, io,
    io::{Read, Seek, SeekFrom},
};

use zerocopy::{FromBytes, Immutable, KnownLayout, LE, U32};

#[derive(Clone, Copy, FromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct FileEntry {
    /// [0, 3] = INI
    /// [1, 0] = MSI
    /// [1, 1] = CAB
    /// [2, 8] = decoder.dll
    /// [3, 7] = 19407F4\\Nutstore.7z
    /// [8, 6] = 19407F4\\FILES.7z
    /// [5, 12] = 1033.dll
    r#type: [U32<LE>; 2],

    xor_flag: U32<LE>,

    /// The file's size in bytes.
    size: U32<LE>,

    /// The file's offset relative to the start of the SFX stub.
    offset: U32<LE>,

    /// Returns the size of the name in UTF-16LE characters.
    name_size: U32<LE>,
}

impl FileEntry {
    pub const fn r#type(&self) -> [u32; 2] {
        [self.r#type[0].get(), self.r#type[1].get()]
    }

    /// Returns the XOR flag.
    #[inline]
    pub const fn xor_flag(&self) -> u32 {
        self.xor_flag.get()
    }

    /// Returns the file's size in bytes.
    #[inline]
    pub const fn size(&self) -> u32 {
        self.size.get()
    }

    /// Returns the offset of the file, relative to the start of the SFX stub.
    #[inline]
    pub const fn offset(&self) -> u32 {
        self.offset.get()
    }

    /// Returns the size of the name in UTF-16LE characters.
    #[inline]
    pub const fn name_size(&self) -> u32 {
        self.name_size.get()
    }

    pub fn read_file<R>(&self, reader: &mut R) -> io::Result<Vec<u8>>
    where
        R: Read + Seek,
    {
        reader.seek(SeekFrom::Start(self.offset().into()))?;

        let mut data = vec![0; self.size() as usize];
        reader.read_exact(&mut data)?;

        if self.xor_flag() == 2 {
            data.iter_mut().take(0x200).for_each(|byte| *byte ^= 0xFF);
        }

        Ok(data)
    }
}

impl fmt::Debug for FileEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileEntry")
            .field("type", &self.r#type())
            .field("xor_flag", &self.xor_flag())
            .field("size", &self.size())
            .field("offset", &self.offset())
            .field("name_size", &self.name_size())
            .finish()
    }
}
