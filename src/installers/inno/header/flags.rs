use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Default)]
    pub struct HeaderFlags: u128 {
        const DISABLE_STARTUP_PROMPT = 1 << 0;
        const CREATE_APP_DIR = 1 << 1;
        const ALLOW_NO_ICONS = 1 << 2;
        const ALWAYS_RESTART = 1 << 3;
        const ALWAYS_USE_PERSONAL_GROUP = 1 << 4;
        const WINDOW_VISIBLE = 1 << 5;
        const WINDOW_SHOW_CAPTION = 1 << 6;
        const WINDOW_RESIZABLE = 1 << 7;
        const WINDOW_START_MAXIMISED = 1 << 8;
        const ENABLED_DIR_DOESNT_EXIST_WARNING = 1 << 9;
        const PASSWORD = 1 << 10;
        const ALLOW_ROOT_DIRECTORY = 1 << 11;
        const DISABLE_FINISHED_PAGE = 1 << 12;
        const CHANGES_ASSOCIATIONS = 1 << 13;
        const USE_PREVIOUS_APP_DIR = 1 << 14;
        const BACK_COLOR_HORIZONTAL = 1 << 15;
        const USE_PREVIOUS_GROUP = 1 << 16;
        const UPDATE_UNINSTALL_LOG_APP_NAME = 1 << 17;
        const USE_PREVIOUS_SETUP_TYPE = 1 << 18;
        const DISABLE_READY_MEMO = 1 << 19;
        const ALWAYS_SHOW_COMPONENTS_LIST = 1 << 20;
        const FLAT_COMPONENTS_LIST = 1 << 21;
        const SHOW_COMPONENT_SIZES = 1 << 22;
        const USE_PREVIOUS_TASKS = 1 << 23;
        const DISABLE_READY_PAGE = 1 << 24;
        const ALWAYS_SHOW_DIR_ON_READY_PAGE = 1 << 25;
        const ALWAYS_SHOW_GROUP_ON_READY_PAGE = 1 << 26;
        const ALLOW_UNC_PATH = 1 << 27;
        const USER_INFO_PAGE = 1 << 28;
        const USE_PREVIOUS_USER_INFO = 1 << 29;
        const UNINSTALL_RESTART_COMPUTER = 1 << 30;
        const RESTART_IF_NEEDED_BY_RUN = 1 << 31;
        const SHOW_TASKS_TREE_LINES = 1 << 32;
        const ALLOW_CANCEL_DURING_INSTALL = 1 << 33;
        const WIZARD_IMAGE_STRETCH = 1 << 34;
        const APPEND_DEFAULT_DIR_NAME = 1 << 35;
        const APPEND_DEFAULT_GROUP_NAME = 1 << 36;
        const ENCRYPTION_USED = 1 << 37;
        const CHANGES_ENVIRONMENT = 1 << 38;
        const SETUP_LOGGING = 1 << 39;
        const SIGNED_UNINSTALLER = 1 << 40;
        const USE_PREVIOUS_LANGUAGE = 1 << 41;
        const DISABLE_WELCOME_PAGE = 1 << 42;
        const CLOSE_APPLICATIONS = 1 << 43;
        const RESTART_APPLICATIONS = 1 << 44;
        const ALLOW_NETWORK_DRIVE = 1 << 45;
        const FORCE_CLOSE_APPLICATIONS = 1 << 46;
        const APP_NAME_HAS_CONSTS = 1 << 47;
        const USE_PREVIOUS_PRIVILEGES = 1 << 48;
        const WIZARD_RESIZABLE = 1 << 49;
        const UNINSTALL_LOGGING = 1 << 50;
        // Obsolete flags
        const UNINSTALLABLE = 1 << 51;
        const DISABLE_DIR_PAGE = 1 << 52;
        const DISABLE_PROGRAM_GROUP_PAGE = 1 << 53;
        const DISABLE_APPEND_DIR = 1 << 54;
        const ADMIN_PRIVILEGES_REQUIRED = 1 << 55;
        const ALWAYS_CREATE_UNINSTALL_ICON = 1 << 56;
        const CREATE_UNINSTALL_REG_KEY = 1 << 57;
        const BZIP_USED = 1 << 58;
        const SHOW_LANGUAGE_DIALOG = 1 << 59;
        const DETECT_LANGUAGE_USING_LOCALE = 1 << 60;
        const DISABLE_DIR_EXISTS_WARNING = 1 << 61;
        const BACK_SOLID = 1 << 62;
        const OVERWRITE_UNINSTALL_REG_ENTRIES = 1 << 63;
        const SHOW_UNDISPLAYABLE_LANGUAGES = 1 << 64;
    }
}

bitflags! {
    /// <https://jrsoftware.org/ishelp/index.php?topic=setup_privilegesrequiredoverridesallowed>
    #[derive(Debug, Default)]
    pub struct PrivilegesRequiredOverrides: u8 {
        const COMMAND_LINE = 1 << 0;
        const DIALOG = 1 << 1;
    }
}
