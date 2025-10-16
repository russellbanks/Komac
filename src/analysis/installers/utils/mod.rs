mod lzma_stream_header;
pub mod registry;

pub use lzma_stream_header::LzmaStreamHeader;

pub const RELATIVE_PROGRAM_FILES_64: &str = "%ProgramFiles%";
pub const RELATIVE_PROGRAM_FILES_32: &str = "%ProgramFiles(x86)%";
pub const RELATIVE_COMMON_FILES_64: &str = "%CommonProgramFiles%";
pub const RELATIVE_COMMON_FILES_32: &str = "%CommonProgramFiles(x86)%";
pub const RELATIVE_LOCAL_APP_DATA: &str = "%LocalAppData%";
pub const RELATIVE_APP_DATA: &str = "%AppData%";
pub const RELATIVE_PROGRAM_DATA: &str = "%ProgramData%";
pub const RELATIVE_WINDOWS_DIR: &str = "%WinDir%";
pub const RELATIVE_SYSTEM_ROOT: &str = "%SystemRoot%";
pub const RELATIVE_SYSTEM_DRIVE: &str = "%SystemDrive%";
pub const RELATIVE_TEMP_FOLDER: &str = "%Temp%";
