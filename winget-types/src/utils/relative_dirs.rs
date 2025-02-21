pub struct RelativeDir;

impl RelativeDir {
    pub const PROGRAM_FILES_64: &'static str = "%ProgramFiles%";
    pub const PROGRAM_FILES_32: &'static str = "%ProgramFiles(x86)%";
    pub const COMMON_FILES_64: &'static str = "%CommonProgramFiles%";
    pub const COMMON_FILES_32: &'static str = "%CommonProgramFiles(x86)%";
    pub const LOCAL_APP_DATA: &'static str = "%LocalAppData%";
    pub const APP_DATA: &'static str = "%AppData%";
    pub const PROGRAM_DATA: &'static str = "%ProgramData%";
    pub const WINDOWS_DIR: &'static str = "%WinDir%";
    pub const SYSTEM_ROOT: &'static str = "%SystemRoot%";
    pub const SYSTEM_DRIVE: &'static str = "%SystemDrive%";
    pub const TEMP_FOLDER: &'static str = "%Temp%";
}
