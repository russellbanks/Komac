use std::{
    fmt, io,
    io::{Read, Seek, SeekFrom},
};

use zerocopy::{FromBytes, Immutable, KnownLayout, LE, U32};

#[derive(Clone, Copy, FromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct FileEntry {
    /// * [0, 3] = INI
    /// * [1, 0] = MSI
    /// * [7, 1], [1, 1] = CAB
    /// * [5, 11], [5, 12] = DLL
    /// * [2, 8] = decoder.dll
    /// * [3, 7] = 7Z
    /// * [8, 6] = FILES.7z
    /// * [1, 13] = AIUI
    /// * [100, 8] = vc_redist.x64.exe
    /// * [101, 8] = vc_redist.x86.exe
    /// * [102, 8] = ndp48-web.exe
    /// * [103, 8] = MicrosoftEdgeWebview2Setup.exe
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

    /// Returns `true` if the file entry is an `msi` file.
    pub fn is_msi(&self) -> bool {
        const MSI: [U32<LE>; 2] = [U32::new(1), U32::ZERO];

        self.r#type == MSI
    }

    /// Returns `true` if the file entry is an `ini` file.
    pub fn is_ini(&self) -> bool {
        const INI: [U32<LE>; 2] = [U32::ZERO, U32::new(3)];

        self.r#type == INI
    }

    /// Returns `true` if the file is a `7z` file.
    pub fn is_7z(&self) -> bool {
        const SEVEN_Z: [U32<LE>; 2] = [U32::new(3), U32::new(7)];

        self.r#type == SEVEN_Z
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
