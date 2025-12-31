mod data_directories;
mod data_directory;
mod standard_fields;
mod windows_fields;

use std::io;

pub use data_directories::DataDirectories;
pub use data_directory::DataDirectory;
pub use standard_fields::{Magic, StandardFields, StandardFields32, StandardFields64};
pub use windows_fields::{WindowsFields, WindowsFields32, WindowsFields64};
use zerocopy::FromBytes;

pub struct OptionalHeader {
    pub standard_fields: StandardFields,
    pub windows_fields: WindowsFields,
    pub data_directories: DataDirectories,
}

impl OptionalHeader {
    pub fn read_from<R>(mut src: R) -> io::Result<Self>
    where
        R: io::Read + io::Seek,
    {
        // Read the optional header magic to determine whether it is 32-bit or 64-bit
        let optional_header_magic = Magic::try_read_from_io(&mut src)?;

        // The StandardFields include the magic so seek back to before the magic
        src.seek_relative(-(size_of::<Magic>() as i64))?;

        // Read the StandardFields
        let standard_fields: StandardFields =
            if optional_header_magic == Magic::ImageNtOptionalHdr64 {
                StandardFields64::try_read_from_io(&mut src)?.into()
            } else {
                StandardFields32::try_read_from_io(&mut src)?.into()
            };

        // Read the WindowsFields
        let windows_fields = if optional_header_magic == Magic::ImageNtOptionalHdr64 {
            WindowsFields64::read_from_io(&mut src)?
        } else {
            WindowsFields32::read_from_io(&mut src)?.into()
        };

        // Read the data directories
        let data_directories = DataDirectories::read_from(
            &mut src,
            windows_fields.number_of_data_directories() as usize,
        )?;

        Ok(Self {
            standard_fields,
            windows_fields,
            data_directories,
        })
    }
}
