use crate::installers::nsis::strings::encoding::nsis_string;
use crate::installers::nsis::version::NsisVersion;
use crate::installers::utils::registry::RegRoot;
use std::borrow::Cow;
use std::ops::Not;
use zerocopy::little_endian::{I32, U32};
use zerocopy::{Immutable, KnownLayout, TryFromBytes};

#[expect(dead_code)]
#[derive(Debug, PartialEq, Eq, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
pub enum Entry {
    Invalid = 0u32.to_le(),
    Return = 1u32.to_le(),
    Jump {
        address: U32,
    } = 2u32.to_le(),
    Abort {
        status: U32,
    } = 3u32.to_le(),
    Quit = 4u32.to_le(),
    Call {
        address: U32, // Encoded as +1
    } = 5u32.to_le(),
    UpdateText {
        update_str: U32,
        ui_st_update_flag: U32,
    } = 6u32.to_le(),
    Sleep {
        time_ms: U32,
    } = 7u32.to_le(),
    BringToFront = 8u32.to_le(),
    ChDetailsView {
        list_action: U32,
        button_action: U32,
    } = 9u32.to_le(),
    SetFileAttributes {
        filename: U32,
        attributes: U32,
    } = 10u32.to_le(),
    CreateDir = 11u32.to_le(),
    IfFileExists = 12u32.to_le(),
    SetFlag {
        id: U32,
        data: U32,
    } = 13u32.to_le(),
    IfFlag {
        on: U32,
        off: U32,
        id: U32,
        new_value_mask: U32,
    } = 14u32.to_le(),
    GetFlag {
        output: U32,
        id: U32,
    } = 15u32.to_le(),
    Rename {
        old: U32,
        new: U32,
        reboot_ok: U32,
    } = 16u32.to_le(),
    GetFullPathname {
        output: U32,
        input: U32,
    } = 17u32.to_le(),
    SearchPath {
        output: U32,
        filename: U32,
    } = 18u32.to_le(),
    GetTempFilename {
        output: U32,
        base_dir: U32,
    } = 19u32.to_le(),
    ExtractFile {
        overwrite_flag: U32,
        name: U32,
        position: U32,
        file_datetime_low: U32,
        file_datetime_high: U32,
        allow_ignore: U32,
    } = 20u32.to_le(),
    DeleteFile {
        filename: U32,
        reboot_ok: U32,
    } = 21u32.to_le(),
    MessageBox {
        mb_flags: U32,
        text: I32,
    } = 22u32.to_le(),
    RemoveDir {
        path: U32,
        recursive_flag: U32,
    } = 23u32.to_le(),
    StrLen {
        output: U32,
        input: U32,
    } = 24u32.to_le(),
    AssignVar {
        variable: U32,
        string_offset: I32,
        max_length: U32,
        start_position: I32,
    } = 25u32.to_le(),
    StrCmp {
        str_1: U32,
        str_2: U32,
        jump_if_equal: U32,
        jump_if_not_equal: U32,
        case_sensitive: U32,
    } = 26u32.to_le(),
    ReadEnv {
        output: U32,
        string_with_env_variables: U32,
        is_read: U32,
    } = 27u32.to_le(),
    IntCmp {
        val_1: U32,
        val_2: U32,
        equal: U32,
        val1_lt_val2: U32,
        val1_gt_val2: U32,
        flags: U32,
    } = 28u32.to_le(),
    IntOp {
        output: U32,
        input1: I32,
        input2: I32,
        operation: U32,
    } = 29u32.to_le(),
    IntFmt {
        output: U32,
        format: U32,
        input: U32,
        _64_bit: U32,
    } = 30u32.to_le(),
    PushPop = 31u32.to_le(),
    FindWindow {
        output_var: U32,
        dialog: U32,
        item_id: U32,
    } = 32u32.to_le(),
    SendMessage {
        output: U32,
        window_handle: U32,
        msg: U32,
        wparam: U32,
        lparam: U32,
    } = 33u32.to_le(),
    IsWindow {
        window_handle: U32,
        jump_if_window: U32,
        jump_if_not_window: U32,
    } = 34u32.to_le(),
    GetDialogItem {
        output_var: U32,
        dialog: U32,
        item_id: U32,
    } = 35u32.to_le(),
    SetCtlColors {
        window_handle: U32,
        pointer_to_struct_colors: U32,
    } = 36u32.to_le(),
    LoadAndSetImage {
        control: U32,
        image_type: U32,
        lr_flags: U32,
        image_id: U32,
        output: U32,
    } = 37u32.to_le(),
    CreateFont {
        handle_output: U32,
        face_name: U32,
        height: U32,
        weight: U32,
        flags: U32,
    } = 38u32.to_le(),
    ShowWindow {
        window_handle: U32,
        show_state: U32,
    } = 39u32.to_le(),
    ShellExec {
        see_mask_flag_: U32,
        verb: U32,
        file: U32,
        parameters: U32,
        show_window: U32,
    } = 40u32.to_le(),
    Execute {
        complete_command_line: U32,
        wait_flag: U32,
        output_error_code: U32,
    } = 41u32.to_le(),
    GetFileTime {
        file: U32,
        high_out: U32,
        low_out: U32,
    } = 42u32.to_le(),
    GetDLLVersion {
        file: U32,
        high_out: U32,
        low_out: U32,
        fixed_offset: U32,
    } = 43u32.to_le(),
    RegisterDLL {
        dll_file_name: U32,
        function_str_ptr: U32,
        display_text: U32,
    } = 44u32.to_le(),
    CreateShortcut {
        link_file: U32,
        target_file: U32,
        parameters: U32,
        icon_file: U32,
        packed_cs_: U32,
    } = 45u32.to_le(),
    CopyFiles {
        source_mask: U32,
        destination_location: U32,
        flags: U32,
    } = 46u32.to_le(),
    Reboot = 47u32.to_le(),
    WriteIni {
        section: U32,
        name: U32,
        value: U32,
        ini_file: U32,
    } = 48u32.to_le(),
    ReadIni {
        output: U32,
        section: U32,
        name: U32,
        ini_file: U32,
    } = 49u32.to_le(),
    DeleteReg {
        root: RegRoot,
        key_name: U32,
        value_name: U32,
        action_and_flags: U32,
    } = 50u32.to_le(),
    WriteReg {
        root: RegRoot,
        key_name: U32,
        value_name: U32,
        value: U32,
        type_len: U32,
    } = 51u32.to_le(),
    ReadRegValue {
        output: U32,
        root: RegRoot,
        key_name: U32,
        item_name: U32,
        one: U32,
    } = 52u32.to_le(),
    RegEnumKey {
        output: U32,
        root_key: RegRoot,
        key_name: U32,
        index: U32,
        value: U32,
    } = 53u32.to_le(),
    FileClose {
        handle: U32,
    } = 54u32.to_le(),
    FileOpen {
        name: U32,
        open_mode: U32,
        create_mode: U32,
        output_handle: U32,
    } = 55u32.to_le(),
    FileWrite {
        handle: U32,
        string: U32,
        int_string: U32,
    } = 56u32.to_le(),
    FileRead {
        handle: U32,
        output: U32,
        max_length: U32,
        get_char_gets: U32,
    } = 57u32.to_le(),
    FileSeek {
        handle: U32,
        offset: U32,
        mode: U32,
        position_output: U32,
    } = 58u32.to_le(),
    FindClose {
        handle: U32,
    } = 59u32.to_le(),
    FindNext {
        output: U32,
        handle: U32,
    } = 60u32.to_le(),
    FindFirst {
        file_spec: U32,
        output: U32,
        handle_output: U32,
    } = 61u32.to_le(),
    WriteUninstaller {
        name: U32,
        offset: U32,
        icon_size: U32,
    } = 62u32.to_le(),
    Log = 63u32.to_le(),
    SectionSet = 64u32.to_le(),
    InstallerType {
        index: U32,
    } = 65u32.to_le(),
    GetLabelAddr = 66u32.to_le(),
    GetFunctionAddr = 67u32.to_le(),
    LockWindow = 68u32.to_le(),
}

impl Entry {
    #[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub fn update_vars<'str_block>(
        &self,
        strings_block: &'str_block [u8],
        user_vars: &mut [Cow<'str_block, str>; 9],
        nsis_version: NsisVersion,
    ) {
        match self {
            Self::GetFullPathname { output, .. } => {
                user_vars[1] = nsis_string(strings_block, output.get(), user_vars, nsis_version);
            }
            Self::SearchPath { filename, .. } => {
                user_vars[0] = nsis_string(strings_block, filename.get(), user_vars, nsis_version);
            }
            Self::GetTempFilename { base_dir, .. } => {
                user_vars[0] = nsis_string(strings_block, base_dir.get(), user_vars, nsis_version);
            }
            Self::ExtractFile { name, .. } => {
                user_vars[0] = nsis_string(strings_block, name.get(), user_vars, nsis_version);
            }
            Self::StrLen { input, .. } => {
                user_vars[0] = nsis_string(strings_block, input.get(), user_vars, nsis_version);
            }
            Self::AssignVar {
                string_offset,
                max_length,
                start_position,
                ..
            } => {
                let result = nsis_string(
                    strings_block,
                    string_offset.get().unsigned_abs(),
                    user_vars,
                    nsis_version,
                );
                let mut start = start_position.get();
                let mut new_len = 0;
                let src_len = result.len() as i32;
                if max_length.get() & !u32::from(u16::MAX) == 0 {
                    new_len = src_len;
                }
                if new_len != 0 {
                    if start < 0 {
                        start += src_len;
                    }

                    start = start.clamp(0, src_len);
                    if start < src_len {
                        user_vars[0] = match result {
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
                user_vars[0] = nsis_string(
                    strings_block,
                    string_with_env_variables.get(),
                    user_vars,
                    nsis_version,
                );
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
                user_vars[0] = Cow::Owned(result.to_string());
            }
            _ => {}
        }
    }
}
