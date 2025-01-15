use crate::installers::nsis::state::NsisState;
use crate::installers::utils::registry::RegRoot;
use std::borrow::Cow;
use std::ops::Not;
use zerocopy::little_endian::I32;
use zerocopy::{Immutable, KnownLayout, TryFromBytes};

#[expect(dead_code)]
#[derive(Debug, PartialEq, Eq, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
pub enum Entry {
    Invalid = 0u32.to_le(),
    Return = 1u32.to_le(),
    Jump {
        address: I32,
    } = 2u32.to_le(),
    Abort {
        status: I32,
    } = 3u32.to_le(),
    Quit = 4u32.to_le(),
    Call {
        address: I32, // Encoded as +1
    } = 5u32.to_le(),
    UpdateText {
        update_str: I32,
        ui_st_update_flag: I32,
    } = 6u32.to_le(),
    Sleep {
        time_ms: I32,
    } = 7u32.to_le(),
    BringToFront = 8u32.to_le(),
    ChDetailsView {
        list_action: I32,
        button_action: I32,
    } = 9u32.to_le(),
    SetFileAttributes {
        filename: I32,
        attributes: I32,
    } = 10u32.to_le(),
    CreateDir = 11u32.to_le(),
    IfFileExists = 12u32.to_le(),
    SetFlag {
        id: I32,
        data: I32,
    } = 13u32.to_le(),
    IfFlag {
        on: I32,
        off: I32,
        id: I32,
        new_value_mask: I32,
    } = 14u32.to_le(),
    GetFlag {
        output: I32,
        id: I32,
    } = 15u32.to_le(),
    Rename {
        old: I32,
        new: I32,
        reboot_ok: I32,
    } = 16u32.to_le(),
    GetFullPathname {
        output: I32,
        input: I32,
    } = 17u32.to_le(),
    SearchPath {
        output: I32,
        filename: I32,
    } = 18u32.to_le(),
    GetTempFilename {
        output: I32,
        base_dir: I32,
    } = 19u32.to_le(),
    ExtractFile {
        overwrite_flag: I32,
        name: I32,
        position: I32,
        file_datetime_low: I32,
        file_datetime_high: I32,
        allow_ignore: I32,
    } = 20u32.to_le(),
    DeleteFile {
        filename: I32,
        reboot_ok: I32,
    } = 21u32.to_le(),
    MessageBox {
        mb_flags: I32,
        text: I32,
    } = 22u32.to_le(),
    RemoveDir {
        path: I32,
        recursive_flag: I32,
    } = 23u32.to_le(),
    StrLen {
        output: I32,
        input: I32,
    } = 24u32.to_le(),
    AssignVar {
        variable: I32,
        string_offset: I32,
        max_length: I32,
        start_position: I32,
    } = 25u32.to_le(),
    StrCmp {
        str_1: I32,
        str_2: I32,
        jump_if_equal: I32,
        jump_if_not_equal: I32,
        case_sensitive: I32,
    } = 26u32.to_le(),
    ReadEnv {
        output: I32,
        string_with_env_variables: I32,
        is_read: I32,
    } = 27u32.to_le(),
    IntCmp {
        val_1: I32,
        val_2: I32,
        equal: I32,
        val1_lt_val2: I32,
        val1_gt_val2: I32,
        flags: I32,
    } = 28u32.to_le(),
    IntOp {
        output: I32,
        input1: I32,
        input2: I32,
        operation: I32,
    } = 29u32.to_le(),
    IntFmt {
        output: I32,
        format: I32,
        input: I32,
        _64_bit: I32,
    } = 30u32.to_le(),
    PushPop = 31u32.to_le(),
    FindWindow {
        output_var: I32,
        dialog: I32,
        item_id: I32,
    } = 32u32.to_le(),
    SendMessage {
        output: I32,
        window_handle: I32,
        msg: I32,
        wparam: I32,
        lparam: I32,
    } = 33u32.to_le(),
    IsWindow {
        window_handle: I32,
        jump_if_window: I32,
        jump_if_not_window: I32,
    } = 34u32.to_le(),
    GetDialogItem {
        output_var: I32,
        dialog: I32,
        item_id: I32,
    } = 35u32.to_le(),
    SetCtlColors {
        window_handle: I32,
        pointer_to_struct_colors: I32,
    } = 36u32.to_le(),
    LoadAndSetImage {
        control: I32,
        image_type: I32,
        lr_flags: I32,
        image_id: I32,
        output: I32,
    } = 37u32.to_le(),
    CreateFont {
        handle_output: I32,
        face_name: I32,
        height: I32,
        weight: I32,
        flags: I32,
    } = 38u32.to_le(),
    ShowWindow {
        window_handle: I32,
        show_state: I32,
    } = 39u32.to_le(),
    ShellExec {
        see_mask_flag_: I32,
        verb: I32,
        file: I32,
        parameters: I32,
        show_window: I32,
    } = 40u32.to_le(),
    Execute {
        complete_command_line: I32,
        wait_flag: I32,
        output_error_code: I32,
    } = 41u32.to_le(),
    GetFileTime {
        file: I32,
        high_out: I32,
        low_out: I32,
    } = 42u32.to_le(),
    GetDLLVersion {
        file: I32,
        high_out: I32,
        low_out: I32,
        fixed_offset: I32,
    } = 43u32.to_le(),
    RegisterDLL {
        dll_file_name: I32,
        function_str_ptr: I32,
        display_text: I32,
    } = 44u32.to_le(),
    CreateShortcut {
        link_file: I32,
        target_file: I32,
        parameters: I32,
        icon_file: I32,
        packed_cs_: I32,
    } = 45u32.to_le(),
    CopyFiles {
        source_mask: I32,
        destination_location: I32,
        flags: I32,
    } = 46u32.to_le(),
    Reboot = 47u32.to_le(),
    WriteIni {
        section: I32,
        name: I32,
        value: I32,
        ini_file: I32,
    } = 48u32.to_le(),
    ReadIni {
        output: I32,
        section: I32,
        name: I32,
        ini_file: I32,
    } = 49u32.to_le(),
    DeleteReg {
        root: RegRoot,
        key_name: I32,
        value_name: I32,
        action_and_flags: I32,
    } = 50u32.to_le(),
    WriteReg {
        root: RegRoot,
        key_name: I32,
        value_name: I32,
        value: I32,
        type_len: I32,
    } = 51u32.to_le(),
    ReadRegValue {
        output: I32,
        root: RegRoot,
        key_name: I32,
        item_name: I32,
        one: I32,
    } = 52u32.to_le(),
    RegEnumKey {
        output: I32,
        root_key: RegRoot,
        key_name: I32,
        index: I32,
        value: I32,
    } = 53u32.to_le(),
    FileClose {
        handle: I32,
    } = 54u32.to_le(),
    FileOpen {
        name: I32,
        open_mode: I32,
        create_mode: I32,
        output_handle: I32,
    } = 55u32.to_le(),
    FileWrite {
        handle: I32,
        string: I32,
        int_string: I32,
    } = 56u32.to_le(),
    FileRead {
        handle: I32,
        output: I32,
        max_length: I32,
        get_char_gets: I32,
    } = 57u32.to_le(),
    FileSeek {
        handle: I32,
        offset: I32,
        mode: I32,
        position_output: I32,
    } = 58u32.to_le(),
    FindClose {
        handle: I32,
    } = 59u32.to_le(),
    FindNext {
        output: I32,
        handle: I32,
    } = 60u32.to_le(),
    FindFirst {
        file_spec: I32,
        output: I32,
        handle_output: I32,
    } = 61u32.to_le(),
    WriteUninstaller {
        name: I32,
        offset: I32,
        icon_size: I32,
    } = 62u32.to_le(),
    Log = 63u32.to_le(),
    SectionSet = 64u32.to_le(),
    InstallerType {
        index: I32,
    } = 65u32.to_le(),
    GetLabelAddr = 66u32.to_le(),
    GetFunctionAddr = 67u32.to_le(),
    LockWindow = 68u32.to_le(),
}

impl Entry {
    #[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub fn update_vars(&self, state: &mut NsisState) {
        match self {
            Self::GetFullPathname { output, .. } => {
                state.user_variables[1] = state.get_string(output.get());
            }
            Self::SearchPath { filename, .. } => {
                state.user_variables[0] = state.get_string(filename.get());
            }
            Self::GetTempFilename { base_dir, .. } => {
                state.user_variables[0] = state.get_string(base_dir.get());
            }
            Self::ExtractFile { name, .. } => {
                state.user_variables[0] = state.get_string(name.get());
            }
            Self::StrLen { input, .. } => {
                state.user_variables[0] = state.get_string(input.get());
            }
            Self::AssignVar {
                string_offset,
                max_length,
                start_position,
                ..
            } => {
                let result = state.get_string(string_offset.get());
                let mut start = start_position.get();
                let new_length = if max_length.get() & !i32::from(u16::MAX) == 0 {
                    result.len()
                } else {
                    0
                };
                if new_length != 0 {
                    if start < 0 {
                        start += result.len() as i32;
                    }

                    let start = u32::try_from(start).unwrap_or_default();
                    if start < result.len() as u32 {
                        state.user_variables[0] = match result {
                            Cow::Borrowed(borrowed) => Cow::Borrowed(&borrowed[start as usize..]),
                            Cow::Owned(mut owned) => {
                                owned.drain(..start as usize);
                                Cow::Owned(owned)
                            }
                        };
                    }
                }
            }
            Self::ReadEnv {
                string_with_env_variables,
                ..
            } => {
                state.user_variables[0] = state.get_string(string_with_env_variables.get());
            }
            Self::IntOp {
                input1,
                input2,
                operation,
                ..
            } => {
                let result = match operation.get() {
                    0 => input1.get() + input2.get(),
                    1 => input1.get() - input2.get(),
                    2 => input1.get() * input2.get(),
                    3 => input1.get() / input2.get(),
                    4 => input1.get() | input2.get(),
                    5 => input1.get() & input2.get(),
                    6 => input1.get() ^ input2.get(),
                    7 => input1.not().get(),
                    8 => i32::from(*input1 != I32::ZERO || *input2 != I32::ZERO),
                    9 => i32::from(*input1 != I32::ZERO && *input2 != I32::ZERO),
                    10 => input1.get() % input2.get(),
                    11 => input1.get().wrapping_shl(input2.get() as u32),
                    12 => input1.get().wrapping_shr(input2.get() as u32),
                    13 => ((input1.get() as u32).wrapping_shr(input2.get() as u32)) as i32,
                    _ => input1.get(),
                };
                state.user_variables[0] = Cow::Owned(result.to_string());
            }
            _ => {}
        }
    }
}
