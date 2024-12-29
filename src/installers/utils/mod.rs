use liblzma::stream::{Filters, Stream};
use std::io::{Error, ErrorKind, Read, Result};

pub mod registry;

pub const RELATIVE_PROGRAM_FILES_64: &str = "%ProgramFiles%";
pub const RELATIVE_PROGRAM_FILES_32: &str = "%ProgramFiles(x86)%";
pub const RELATIVE_COMMON_FILES_64: &str = "%CommonProgramFiles%";
pub const RELATIVE_COMMON_FILES_32: &str = "%CommonProgramFiles(x86)%";
pub const RELATIVE_LOCAL_APP_DATA: &str = "%LocalAppData%";
pub const RELATIVE_APP_DATA: &str = "%AppData%";
pub const RELATIVE_PROGRAM_DATA: &str = "%ProgramData%";
pub const RELATIVE_WINDOWS_DIR: &str = "%WinDir%";
pub const RELATIVE_SYSTEM_ROOT: &str = "%SystemRoot%";
pub const RELATIVE_TEMP_FOLDER: &str = "%Temp%";

pub fn read_lzma_stream_header<R: Read>(reader: &mut R) -> Result<Stream> {
    let mut properties = [0; 5];
    reader.read_exact(&mut properties)?;

    let mut filters = Filters::new();
    filters.lzma1_properties(&properties)?;

    Stream::new_raw_decoder(&filters).map_err(|error| Error::new(ErrorKind::InvalidData, error))
}
