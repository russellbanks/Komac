use super::super::{
    super::utils::{
        RELATIVE_APP_DATA, RELATIVE_COMMON_FILES_32, RELATIVE_COMMON_FILES_64,
        RELATIVE_LOCAL_APP_DATA, RELATIVE_PROGRAM_FILES_32, RELATIVE_PROGRAM_FILES_64,
        RELATIVE_SYSTEM_ROOT, RELATIVE_WINDOWS_DIR,
    },
    state::NsisState,
};

/// NSIS can use one name for two CSIDL_*** and `CSIDL_COMMON`_*** items (`CurrentUser` / `AllUsers`)
/// Some NSIS shell names are not identical to WIN32 CSIDL_* names.
/// NSIS doesn't use some CSIDL_* values.
///
/// Some values have been adapted to use relative folders for winget.
const STRINGS: &[Option<&str>; 62] = &[
    Some("Desktop"),
    Some("Internet"),
    Some("SMPrograms"),
    Some("Controls"),
    Some("Printers"),
    Some("Documents"),
    Some("Favorites"),
    Some("SMStartup"),
    Some("Recent"),
    Some("SendTo"),
    Some("BitBucket"),
    Some("StartMenu"),
    None,
    Some("Music"),
    Some("Videos"),
    None,
    Some("Desktop"),
    Some("Drives"),
    Some("Network"),
    Some("NetHood"),
    Some("Fonts"),
    Some("Templates"),
    Some("StartMenu"),
    Some("SMPrograms"),
    Some("SMStartup"),
    Some("Desktop"),
    Some(RELATIVE_APP_DATA),
    Some("PrintHood"),
    Some(RELATIVE_LOCAL_APP_DATA),
    Some("ALTStartUp"),
    Some("ALTStartUp"),
    Some("Favorites"),
    Some("InternetCache"),
    Some("Cookies"),
    Some("History"),
    Some(RELATIVE_APP_DATA),
    Some(RELATIVE_WINDOWS_DIR),
    Some(RELATIVE_SYSTEM_ROOT),
    Some(RELATIVE_PROGRAM_FILES_64),
    Some("Pictures"),
    Some("Profile"),
    Some("System32"),
    Some(RELATIVE_PROGRAM_FILES_32),
    Some(RELATIVE_COMMON_FILES_64),
    Some(RELATIVE_COMMON_FILES_32),
    Some("Templates"),
    Some("Documents"),
    Some("AdminTools"),
    Some("AdminTools"),
    Some("Connections"),
    None,
    None,
    None,
    Some("Music"),
    Some("Pictures"),
    Some("Videos"),
    Some("Resources"),
    Some("ResourcesLocalized"),
    Some("CommonOEMLinks"),
    Some("CDBurnArea"),
    None,
    Some("ComputersNearMe"),
];

pub struct Shell;

impl Shell {
    /// Adapted from <https://github.com/mcmilk/7-Zip/blob/HEAD/CPP/7zip/Archive/Nsis/NsisIn.cpp#L683>
    pub fn resolve(buf: &mut String, state: &NsisState, character: u16) {
        const PROGRAM_FILES_DIR: &str = "ProgramFilesDir";
        const COMMON_FILES_DIR: &str = "CommonFilesDir";

        let (index1, index2): (u8, u8) = character.to_le_bytes().into();

        if index1 & (1 << 7) != 0 {
            let offset = index1 & 0x3F;
            let is_64_bit = index1 & (1 << 6) != 0;
            let shell_string = state.get_string(i32::from(offset));
            if shell_string == PROGRAM_FILES_DIR {
                buf.push_str(if is_64_bit {
                    RELATIVE_PROGRAM_FILES_64
                } else {
                    RELATIVE_PROGRAM_FILES_32
                });
            } else if shell_string == COMMON_FILES_DIR {
                buf.push_str(if is_64_bit {
                    RELATIVE_COMMON_FILES_64
                } else {
                    RELATIVE_COMMON_FILES_32
                });
            }
        } else if let Some(Some(shell_str)) = STRINGS.get(index1 as usize) {
            buf.push_str(shell_str);
        } else if let Some(Some(shell_str)) = STRINGS.get(index2 as usize) {
            buf.push_str(shell_str);
        }
    }
}
