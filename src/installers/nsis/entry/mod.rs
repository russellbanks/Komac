mod creation_disposition;
mod exec_flag;
mod generic_access_rights;
mod push_pop;
mod seek_from;
mod show_window;
mod window_message;

use std::{
    borrow::Cow,
    cmp::{Ordering, max},
    ops::Not,
};

use compact_str::{ToCompactString, format_compact};
use nt_time::FileTime;
use thiserror::Error;
use tracing::debug;
use zerocopy::{
    Immutable, KnownLayout, TryFromBytes,
    little_endian::{I32, U16, U64},
    transmute_ref,
};

use super::{
    entry::{
        creation_disposition::CreationDisposition, exec_flag::ExecFlag,
        generic_access_rights::GenericAccessRights, push_pop::PushPop, seek_from::SeekFrom,
        show_window::ShowWindow, window_message::WindowMessage,
    },
    registry::RegType,
    state::NsisState,
};
use crate::installers::{nsis::file_system::RelativeLocation, utils::registry::RegRoot};

#[derive(Debug, Error)]
#[error("Reached invalid entry")]
pub struct EntryError;

#[expect(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromBytes, KnownLayout, Immutable)]
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
    CreateDir {
        path: I32,
        update_install_dir: I32,
        restrict_ac_1: I32,
    } = 11u32.to_le(),
    IfFileExists {
        file_name: I32,
        jump_amount_if_exists: I32,
        jump_amount_if_not_exists: I32,
    } = 12u32.to_le(),
    SetFlag {
        id: I32,
        data: I32,
    } = 13u32.to_le(),
    IfFlag {
        on: I32,
        off: I32,
        r#type: ExecFlag,
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
        long_or_short_file_name: I32,
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
        datetime: U64,
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
        recursive: I32,
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
        is_64_bit: I32,
    } = 30u32.to_le(),
    PushPop {
        variable_or_string: I32,
        push_pop: PushPop,
        exchange: I32,
    } = 31u32.to_le(),
    FindWindow {
        output: I32,
        window_class: I32,
        window_name: I32,
        window_parent: I32,
        window_after: I32,
    } = 32u32.to_le(),
    SendMessage {
        output: I32,
        handle: I32,
        msg: I32,
        wide_param: I32,
        long_param: I32,
    } = 33u32.to_le(),
    IsWindow {
        window_handle: I32,
        jump_if_window: I32,
        jump_if_not_window: I32,
    } = 34u32.to_le(),
    GetDialogItem {
        output: I32,
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
        hide_window: I32,
        enable_window: I32,
    } = 39u32.to_le(),
    ShellExec {
        see_mask_flag: I32,
        verb: I32,
        file: I32,
        parameters: I32,
        show_window: I32,
        status_text: I32,
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
        no_unload: I32,
    } = 44u32.to_le(),
    CreateShortcut {
        link_file: I32,
        target_file: I32,
        parameters: I32,
        icon_file: I32,
        create_shortcut: I32,
    } = 45u32.to_le(),
    CopyFiles {
        source_mask: I32,
        destination_location: I32,
        flags: I32,
        status_text: I32,
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
        reserved: I32,
        root: RegRoot,
        key_name: I32,
        value_name: I32,
        flags: I32,
    } = 50u32.to_le(),
    WriteReg {
        root: RegRoot,
        key_name: I32,
        value_name: I32,
        value: I32,
        r#type: RegType,
        sub_type: RegType,
    } = 51u32.to_le(),
    ReadReg {
        output: I32,
        root: RegRoot,
        key_name: I32,
        value_name: I32,
        r#type: I32, // DWORD if == 1 else Str
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
        open_mode: GenericAccessRights,
        create_mode: CreationDisposition,
        output_handle: I32,
    } = 55u32.to_le(),
    FileWrite {
        handle: I32,
        string: I32,
        int_or_string: I32,
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
        seek_from: SeekFrom,
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
    SectionSet {
        index: I32,
        output: I32,
        r#type: I32,
        call_section_set_flags: I32,
        output_2: I32,
    } = 63u32.to_le(),
    InstallerTypeSet {
        index: I32,
    } = 64u32.to_le(),
    GetOSInfo = 65u32.to_le(),
    ReservedOPCode = 66u32.to_le(),
    LockWindow {
        on_off: I32,
    } = 67u32.to_le(),
    FileWriteUTF16LE {
        handle: I32,
        string: I32,
        int_or_string: I32,
    } = 68u32.to_le(),
    FileReadUTF16LE {
        handle: I32,
        output: I32,
        max_length: I32,
        get_char_gets: I32,
    } = 69u32.to_le(),
    Log {
        set: I32,
        text: I32,
    } = 70u32.to_le(),
    FindProcess = 71u32.to_le(),
    GetFontVersion = 72u32.to_le(),
    GetFontName = 73u32.to_le(),
}

impl Entry {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cognitive_complexity
    )]
    pub fn execute(&self, state: &mut NsisState) -> Result<i32, EntryError> {
        match self {
            Self::Invalid => {
                debug!("Invalid");
                return Err(EntryError);
            }
            Self::Return => {
                debug!("Return");
            }
            Self::Jump { address } => {
                debug!("Jump: {address}");
                return Ok(address.get());
            }
            Self::Abort { status } => {
                debug!(r#"Aborting: "{}""#, state.get_string(status.get()));
            }
            Self::Quit => {
                debug!("Quit");
            }
            Self::Call { address } => {
                let resolved_address = state.resolve_address(address.get()) - 1;
                debug!("Call: {resolved_address}");
                return state.execute_code_segment(resolved_address);
            }
            Self::UpdateText { update_str, .. } => {
                debug!("DetailPrint: {}", state.get_string(update_str.get()));
            }
            Self::Sleep { time_ms } => {
                debug!("Sleep: {}", max(state.get_int(time_ms.get()), 1));
            }
            Self::BringToFront => {
                debug!("BringToFront");
            }
            Self::ChDetailsView {
                list_action,
                button_action,
            } => {
                debug!("ChDetailsView: {list_action} {button_action}");
            }
            Self::SetFileAttributes {
                filename,
                attributes,
            } => {
                let filename = state.get_string(filename.get());
                debug!(
                    r#"SetFileAttributes: "{filename}":{:#08X}"#,
                    attributes.get()
                );
            }
            Self::CreateDir {
                path,
                update_install_dir,
                restrict_ac_1: _restrict_ac_1,
            } => {
                let path = state.get_string(path.get());
                if *update_install_dir == I32::ZERO {
                    debug!(r#"CreateDirectory: "{path}""#);
                    state
                        .file_system
                        .create_directory(&*path, RelativeLocation::Current);
                } else {
                    debug!(r#"SetOutPath: "{path}""#);
                    state
                        .file_system
                        .set_directory(&*path, RelativeLocation::Root);
                }
            }
            Self::IfFileExists {
                file_name,
                jump_amount_if_exists,
                jump_amount_if_not_exists,
            } => {
                let file_name = state.get_string(file_name.get());
                debug!(
                    r#"IfFileExists: "{file_name}" if exists {jump_amount_if_exists} if not exists {jump_amount_if_not_exists}"#
                );

                // Because we're only simulating the control flow, assume the file does not exist
                return Ok(jump_amount_if_not_exists.get());
            }
            Self::SetFlag { id, data } => {
                debug!("SetFlag: {id} {data}");
            }
            Self::IfFlag {
                on,
                off,
                r#type,
                new_value_mask: _new_value_mask,
            } => {
                debug!("If{type}Flag: {on} {off}");
            }
            Self::GetFlag { output, id } => {
                debug!("GetFlag: {id} {output}");
            }
            Self::Rename {
                old,
                new,
                reboot_ok,
            } => {
                let old = state.get_string(old.get());
                let new = state.get_string(new.get());
                let reboot_ok = state.get_string(reboot_ok.get());
                debug!("Rename: {old} {new} {reboot_ok}");
            }
            Self::GetFullPathname {
                output,
                input,
                long_or_short_file_name,
            } => {
                // https://github.com/mcmilk/7-Zip/blob/HEAD/CPP/7zip/Archive/Nsis/NsisIn.cpp#L3986

                debug!(
                    "GetFullPathName: {}{} {}",
                    if *long_or_short_file_name == I32::ZERO {
                        "/SHORT "
                    } else {
                        ""
                    },
                    state
                        .variables
                        .get(&(input.get() as usize))
                        .map(|input| &**input)
                        .unwrap_or_default(),
                    state.get_string(output.get())
                );
            }
            Self::SearchPath { output, filename } => {
                debug!(
                    "SearchPath: {} {}",
                    state
                        .variables
                        .get(&(output.get() as usize))
                        .map(|input| &**input)
                        .unwrap_or_default(),
                    state.get_string(filename.get())
                );
            }
            Self::GetTempFilename { output, base_dir } => {
                debug!(
                    "GetTempFilename: {} {}",
                    state
                        .variables
                        .get(&(output.get() as usize))
                        .map(|output| &**output)
                        .unwrap_or_default(),
                    state.get_string(base_dir.get())
                );
            }
            Self::ExtractFile {
                overwrite_flag: _overwrite_flag,
                name,
                position: _position,
                datetime,
                allow_ignore: _allow_ignore,
            } => {
                let name = state.get_string(name.get());
                let date = if *datetime == U64::MAX_VALUE {
                    FileTime::NT_TIME_EPOCH
                } else {
                    FileTime::new(datetime.get())
                };
                debug!(r#"ExtractFile: "{name}" {date}"#);
                state.file_system.create_file(&*name, date);
            }
            Self::DeleteFile {
                filename,
                reboot_ok: _reboot_ok,
            } => {
                let filename = state.get_string(filename.get());
                debug!(r#"Delete: "{filename}""#);
                state.file_system.delete_file(&*filename);
            }
            Self::MessageBox { mb_flags, text } => {
                let text = state.get_string(text.get());
                debug!(r#"MessageBox: {}, "{text}""#, mb_flags);
            }
            Self::RemoveDir { path, recursive } => {
                let path = state.get_string(path.get());
                debug!(r#"RMDir: "{path}", recursive={recursive}"#);
            }
            Self::StrLen { output, input } => {
                let input = state.get_string(input.get());
                debug!(r#"StrLen: "{input}".len() = {}"#, input.len());
                state.variables.insert(
                    output.get().unsigned_abs() as usize,
                    Cow::Owned(input.len().to_string()),
                );
            }
            Self::AssignVar {
                variable,
                string_offset,
                max_length,
                start_position,
            } => {
                let mut result = state.get_string(string_offset.get());
                let mut start = start_position.get();
                let [low, high]: &[U16; 2] = transmute_ref!(max_length);
                let new_length = if high == &U16::ZERO {
                    result.len()
                } else {
                    usize::from(low.get())
                };
                if new_length > 0 {
                    if start < 0 {
                        start += result.len() as i32;
                    }

                    let start = u32::try_from(start).unwrap_or_default();
                    if start < result.len() as u32 {
                        let index = variable.get().unsigned_abs() as usize;
                        result = match result {
                            Cow::Borrowed(borrowed) => Cow::Borrowed(&borrowed[start as usize..]),
                            Cow::Owned(mut owned) => {
                                owned.drain(..start as usize);
                                Cow::Owned(owned)
                            }
                        };
                        debug!(r#"AssignVar: {index} "{result}""#);
                        state
                            .variables
                            .insert(variable.get().unsigned_abs() as usize, result);
                    }
                } else {
                    state
                        .variables
                        .remove(&(variable.get().unsigned_abs() as usize));
                }
            }
            Self::StrCmp {
                str_1,
                str_2,
                jump_if_equal,
                jump_if_not_equal,
                case_sensitive,
            } => {
                let str_1 = state.get_string(str_1.get());
                let str_2 = state.get_string(str_2.get());

                debug!(
                    r#"StrCmp: "{str_1}" "{str_2}" eq: {jump_if_equal} ne: {jump_if_not_equal}"#
                );

                let equal = match *case_sensitive {
                    I32::ZERO => str_1.eq_ignore_ascii_case(&str_2), // Case-insensitive
                    _ => str_1 == str_2,                             // Case-sensitive
                };

                return if equal {
                    Ok(jump_if_equal.get())
                } else {
                    Ok(jump_if_not_equal.get())
                };
            }
            Self::ReadEnv {
                output,
                string_with_env_variables,
                ..
            } => {
                let output = output.get().unsigned_abs() as usize;
                let env_string = state.get_string(string_with_env_variables.get());
                debug!("ReadEnv: {output} {env_string}");
                state
                    .variables
                    .insert(output, state.get_string(string_with_env_variables.get()));
            }
            Self::IntCmp {
                val_1,
                val_2,
                equal,
                val1_lt_val2,
                val1_gt_val2,
                flags,
            } => {
                let signed = *flags == I32::ZERO;
                let is_64_bit = (*flags & I32::new(0x8000)) != 0;

                let val_1 = state.get_int(val_1.get());
                let val_2 = state.get_int(val_2.get());
                let comparison = match (signed, is_64_bit) {
                    (true, true) => i64::from(val_1).cmp(&i64::from(val_2)),
                    (true, false) => val_1.cmp(&val_2),
                    (false, true) => (val_1 as u64).cmp(&(val_2 as u64)),
                    (false, false) => (val_1 as u32).cmp(&(val_2 as u32)),
                };

                debug!(
                    "IntCmp{}: {val_1} {val_2} eq: {} lt: {} gt: {}",
                    if is_64_bit { "64" } else { "" },
                    state.resolve_address(equal.get()),
                    state.resolve_address(val1_lt_val2.get()),
                    state.resolve_address(val1_gt_val2.get())
                );

                return match comparison {
                    Ordering::Less => Ok(val1_lt_val2.get()),
                    Ordering::Greater => Ok(val1_gt_val2.get()),
                    Ordering::Equal => Ok(equal.get()),
                };
            }
            Self::IntOp {
                output,
                input1,
                input2,
                operation,
            } => {
                const SIGNS: [&str; 13] = [
                    "+", "-", "*", "/", "|", "&", "^", "!", "||", "&&", "%", "<<", ">>",
                ];

                let input1 = state.get_int(input1.get());
                let input2 = state.get_int(input2.get());

                debug!(
                    "IntOp: {input1} {} {input2}",
                    SIGNS.get(operation.get() as usize).map_or("@", |sign| sign),
                );

                #[expect(clippy::cast_sign_loss)]
                let result = match operation.get() {
                    0 => input1 + input2,
                    1 => input1 - input2,
                    2 => input1 * input2,
                    3 => input1 / input2,
                    4 => input1 | input2,
                    5 => input1 & input2,
                    6 => input1 ^ input2,
                    7 => input1.not(),
                    8 => i32::from(input1 != 0 || input2 != 0),
                    9 => i32::from(input1 != 0 && input2 != 0),
                    10 => input1 % input2,
                    11 => input1 << input2 as u32,
                    12 => input1 >> input2 as u32,
                    13 => ((input1 as u32) >> (input2 as u32)) as i32,
                    _ => input1,
                };
                state.variables.insert(
                    output.get().unsigned_abs() as usize,
                    Cow::Owned(result.to_string()),
                );
            }
            Self::IntFmt {
                output,
                format,
                input,
                is_64_bit,
            } => {
                let is_64_bit = *is_64_bit != I32::ZERO;
                let format = state.get_string(format.get());
                let formatted = if format.starts_with("0x") {
                    format_compact!("{:#X}", input.get())
                } else {
                    input.get().to_compact_string()
                };
                debug!("IntFmt{}: {formatted}", if is_64_bit { "64" } else { "" });
                state
                    .variables
                    .insert(output.get().unsigned_abs() as usize, formatted.into());
            }
            Self::PushPop {
                variable_or_string,
                push_pop,
                exchange,
            } => {
                if *exchange != I32::ZERO {
                    let count = exchange.get().unsigned_abs() as usize;
                    if state.stack.len() > count {
                        state.stack.swap(0, count);
                    }
                    if *exchange == I32::ZERO {
                        debug!("Exchange");
                    } else {
                        debug!("Exchange: {exchange}");
                    }
                } else if *push_pop == PushPop::Pop {
                    if let Some(variable) = state.stack.pop() {
                        debug!(r#"Pop: "{variable}""#);
                        state
                            .variables
                            .insert(variable_or_string.get().unsigned_abs() as usize, variable);
                    }
                } else if *push_pop == PushPop::Push {
                    let string = state.get_string(variable_or_string.get());
                    debug!(r#"Push: "{string}""#);
                    state.stack.push(string);
                }
            }
            Self::FindWindow {
                output: _output,
                window_class,
                window_name,
                window_parent,
                window_after: _window_after,
            } => {
                debug!(
                    r#"FindWindow: "{}" "{}" {}"#,
                    state.get_string(window_class.get()),
                    state.get_string(window_name.get()),
                    state.get_string(window_parent.get())
                );
            }
            Self::SendMessage {
                output: _output,
                handle: _handle,
                msg,
                wide_param,
                long_param,
            } => {
                let window_message = WindowMessage::try_from(state.get_int(msg.get()) as u16);

                let wide = state.get_string(wide_param.get());
                let long = state.get_string(long_param.get());

                if let Ok(window_message) = window_message {
                    debug!(r#"SendMessage: {window_message} "{wide}" "{long}""#);
                } else {
                    debug!(
                        r#"SendMessage: {} "{wide}" "{long}""#,
                        state.get_string(msg.get())
                    );
                }
            }
            Self::IsWindow {
                window_handle,
                jump_if_window,
                jump_if_not_window,
            } => {
                debug!(
                    "IsWindow: {window_handle} window: {jump_if_window} not_window: {jump_if_not_window}"
                );
            }
            Self::GetDialogItem {
                output: _output,
                dialog: _dialog,
                item_id,
            } => {
                debug!("GetDialogItem: {}", state.get_int(item_id.get()));
            }
            Self::SetCtlColors {
                window_handle,
                pointer_to_struct_colors,
            } => {
                debug!("SetCtlColors: {window_handle} {pointer_to_struct_colors}");
            }
            Self::LoadAndSetImage {
                control,
                image_type,
                lr_flags,
                image_id,
                output,
            } => {
                debug!("LoadAndSetImage: {control} {image_type} {lr_flags} {image_id} {output}");
            }
            Self::CreateFont {
                handle_output,
                face_name,
                height,
                weight,
                flags,
            } => {
                debug!("CreateFont: {handle_output} {face_name} {height} {weight} {flags}");
            }
            Self::ShowWindow {
                window_handle: _window_handle,
                show_state,
                hide_window,
                enable_window,
            } => {
                // https://github.com/kichik/nsis/blob/HEAD/Source/exehead/exec.c#L892
                if *enable_window != I32::ZERO {
                    debug!("EnableWindow");
                } else if *hide_window != I32::ZERO {
                    debug!("HideWindow");
                } else if let Ok(show_window) =
                    ShowWindow::try_from(state.get_int(show_state.get()))
                {
                    debug!("ShowWindow: {show_window}");
                }
            }
            Self::ShellExec {
                verb,
                file,
                parameters,
                show_window,
                see_mask_flag: _see_mask_flag,
                status_text,
            } => {
                debug!(
                    "ShellExec: {verb} {file} {parameters} {show_window} {status_text}",
                    verb = state.get_string(verb.get()),
                    file = state.get_string(file.get()),
                    parameters = state.get_string(parameters.get()),
                    show_window = ShowWindow::try_from(state.get_int(show_window.get()))
                        .map(ShowWindow::as_str)
                        .unwrap_or_default(),
                    status_text = state.get_string(status_text.get()),
                );
            }
            Self::Execute {
                complete_command_line,
                wait_flag,
                output_error_code,
            } => {
                debug!("Execute: {complete_command_line} {wait_flag} {output_error_code}");
            }
            Self::GetFileTime {
                file,
                high_out,
                low_out,
            } => {
                debug!("GetFileTime: {file} {high_out} {low_out}");
            }
            Self::GetDLLVersion {
                file,
                high_out,
                low_out,
                fixed_offset,
            } => {
                debug!("GetDLLVersion: {file} {high_out} {low_out} {fixed_offset}");
            }
            Self::RegisterDLL {
                dll_file_name,
                function_str_ptr,
                display_text,
                no_unload,
            } => {
                // https://github.com/mcmilk/7-Zip/blob/HEAD/CPP/7zip/Archive/Nsis/NsisIn.cpp#L4398

                let dll_file_name = state.get_string(dll_file_name.get());
                let function_str_ptr = state.get_string(function_str_ptr.get());
                let display_text = state.get_string(display_text.get());
                debug!(
                    "RegisterDLL: {dll_file_name} {function_str_ptr} {display_text}{}",
                    if no_unload.get() == 1 {
                        " /NOUNLOAD"
                    } else {
                        ""
                    }
                );
            }
            Self::CreateShortcut {
                link_file,
                target_file,
                parameters: _parameters,
                icon_file,
                create_shortcut,
            } => {
                // <https://github.com/kichik/nsis/blob/HEAD/Source/exehead/fileform.h#L559>
                const NO_WORKING_DIRECTORY: I32 = I32::new(0x8000);

                // https://github.com/kichik/nsis/blob/HEAD/Source/exehead/exec.c#L1087
                let link_file = state.get_string(link_file.get());
                let target_file = state.get_string(target_file.get());
                let icon_file = state.get_string(icon_file.get());

                debug!(
                    r#"CreateShortcut: {}out: "{link_file}", in: "{target_file}", icon: "{icon_file}""#,
                    if *create_shortcut & NO_WORKING_DIRECTORY != 0 {
                        "/NoWorkingDir "
                    } else {
                        ""
                    },
                );
            }
            Self::CopyFiles {
                source_mask,
                destination_location,
                flags,
                status_text,
            } => {
                // https://github.com/kichik/nsis/blob/HEAD/Source/exehead/exec.c#L1152
                // https://github.com/mcmilk/7-Zip/blob/HEAD/CPP/7zip/Archive/Nsis/NsisIn.cpp#L4505
                debug!(
                    r#"CopyFiles {}{}"{}" -> "{}" {}"#,
                    if flags.get() & 0x04 != 0 {
                        "/SILENT "
                    } else {
                        ""
                    },
                    if flags.get() & 0x80 != 0 {
                        "/FILESONLY "
                    } else {
                        ""
                    },
                    state.get_string(source_mask.get()),
                    state.get_string(destination_location.get()),
                    state.get_string(status_text.get()),
                );
            }
            Self::Reboot => {
                debug!("Reboot");
            }
            Self::WriteIni {
                section,
                name,
                value,
                ini_file,
            } => {
                debug!("WriteIni: {section} {name} {value} {ini_file}");
            }
            Self::ReadIni {
                output,
                section,
                name,
                ini_file,
            } => {
                let name = state.get_string(name.get());
                let section = state.get_string(section.get());
                let ini_file = state.get_string(ini_file.get());
                debug!("ReadIni: {output} {name} {section} {ini_file}");
            }
            Self::DeleteReg {
                reserved: _reserved,
                root,
                key_name,
                value_name,
                flags,
            } => {
                // https://github.com/kichik/nsis/blob/HEAD/Source/exehead/exec.c#L1240
                // https://github.com/mcmilk/7-Zip/blob/HEAD/CPP/7zip/Archive/Nsis/NsisIn.cpp#L4558

                let key_name = state.get_string(key_name.get());

                if *flags == I32::ZERO {
                    let value_name = state.get_string(value_name.get());

                    debug!(r#"DeleteRegValue: "{root}\{key_name}" "{value_name}""#);

                    state
                        .registry
                        .remove_value_name(*root, &key_name, &value_name);
                } else {
                    debug!(
                        r#"DeleteRegKey: {}"{root}\{key_name}""#,
                        if flags.get() & 2 != 0 {
                            "/ifempty "
                        } else {
                            ""
                        }
                    );

                    state.registry.remove_key(*root, &key_name);
                }
            }
            Self::WriteReg {
                root,
                key_name,
                value_name,
                value,
                r#type,
                sub_type,
            } => {
                // https://github.com/kichik/nsis/blob/HEAD/Source/exehead/exec.c#L1265

                let key_name = state.get_string(key_name.get());
                let value_name = state.get_string(value_name.get());

                if *r#type == RegType::String {
                    let value = state.get_string(value.get());

                    if *sub_type == RegType::String {
                        debug!(r#"WriteRegStr: "{root}\{key_name}" "{value_name}"="{value}""#);
                    } else {
                        debug!(
                            r#"WriteRegExpandStr: "{root}\{key_name}" "{value_name}"="{value}""#
                        );
                    }

                    state
                        .registry
                        .insert_value(*root, key_name, value_name, value);
                } else if *r#type == RegType::DWord {
                    let value = state.get_int(value.get());

                    debug!(r#"WriteRegDWORD: "{root}\{key_name}" "{value_name}"="{value}""#);

                    state.registry.insert_value(
                        *root,
                        key_name,
                        value_name,
                        value.to_compact_string(),
                    );
                } else if *r#type == RegType::Binary {
                    debug!(
                        r#"WriteReg{}: "{root}\{key_name}" "{value_name}"="{{BINARY DATA}}""#,
                        match *sub_type {
                            RegType::None => "None",
                            RegType::MultiString => "MultiStr",
                            _ => "Bin",
                        }
                    );
                }
            }
            Self::ReadReg {
                output,
                root,
                key_name,
                value_name,
                r#type,
            } => {
                let key_name = state.get_string(key_name.get());
                let value_name = state.get_string(value_name.get());

                let output = state.get_int(output.get());
                debug!(
                    "ReadReg{}: {output} {root} {key_name} {value_name}",
                    if r#type.get() == 1 { "Dword" } else { "Str" }
                );
            }
            Self::RegEnumKey {
                output,
                root_key,
                key_name,
                index,
                value,
            } => {
                debug!("RegEnumKey: {output} {root_key} {key_name} {index} {value}");
            }
            Self::FileClose { handle } => {
                let handle = state.get_int(handle.get());
                debug!("FileClose: {handle}");
            }
            Self::FileOpen {
                name,
                open_mode,
                create_mode,
                output_handle,
            } => {
                // https://github.com/mcmilk/7-Zip/blob/HEAD/CPP/7zip/Archive/Nsis/NsisIn.cpp#L4688
                let name = state
                    .variables
                    .get(&(name.get() as usize))
                    .map(|name| &**name)
                    .unwrap_or_default();
                let output_handle = state.get_string(output_handle.get());
                let open_option = match (*create_mode, *open_mode) {
                    (CreationDisposition::OpenExisting, GenericAccessRights::GENERIC_READ) => {
                        "READ"
                    }
                    (CreationDisposition::CreateAlways, GenericAccessRights::GENERIC_WRITE) => {
                        "WRITE"
                    }
                    (
                        CreationDisposition::OpenAlways,
                        GenericAccessRights::GENERIC_WRITE | GenericAccessRights::GENERIC_READ,
                    ) => "APPEND",
                    _ => "",
                };
                debug!(r#"FileOpen: "{name}" {output_handle} {open_option}"#);
            }
            Self::FileWrite {
                handle,
                string,
                int_or_string,
            } => {
                debug!("FileWrite: {handle} {string} {int_or_string}");
            }
            Self::FileRead {
                handle,
                output,
                max_length,
                get_char_gets,
            } => {
                debug!("FileRead: {handle} {output} {max_length} {get_char_gets}");
            }
            Self::FileSeek {
                handle,
                offset,
                mode,
                seek_from,
            } => {
                let handle = state
                    .variables
                    .get(&(handle.get() as usize))
                    .map(|handle| &**handle)
                    .unwrap_or_default();
                debug!(
                    "FileSeek: {handle} {} {seek_from} {offset}",
                    state.get_int(mode.get())
                );
            }
            Self::FindClose { handle } => {
                debug!("FindClose: {handle}");
            }
            Self::FindNext { output, handle } => {
                let output = state
                    .variables
                    .get(&(output.get() as usize))
                    .map(|output| &**output)
                    .unwrap_or_default();
                let handle = state
                    .variables
                    .get(&(handle.get() as usize))
                    .and_then(|handle| handle.parse::<i32>().ok())
                    .unwrap_or_default();
                debug!("FindNext: {output} {handle}");
            }
            Self::FindFirst {
                file_spec,
                output,
                handle_output,
            } => {
                debug!("FindFirst: {file_spec} {output} {handle_output}");
            }
            Self::WriteUninstaller {
                name,
                offset,
                icon_size,
            } => {
                let name = state.get_string(name.get());
                debug!(r#"WriteUninstaller: "{name}" {offset} {icon_size}"#);
            }
            Self::SectionSet {
                index,
                output,
                r#type,
                call_section_set_flags: _call_section_set_flags,
                output_2,
            } => {
                const SECTION_NAMES: [&str; 6] =
                    ["Text", "InstTypes", "Flags", "Code", "CodeSize", "Size"];

                let index = state.get_int(index.get());
                if let Ok(r#type) = u32::try_from(r#type.get()) {
                    debug!(
                        "SectionGet{}: {index} {output}",
                        SECTION_NAMES[r#type as usize]
                    );
                } else {
                    let r#type = r#type.get().unsigned_abs() - 1;
                    debug!(
                        "SectionSet{}: {index} {}",
                        SECTION_NAMES[r#type as usize],
                        state.get_int(if r#type == 0 { output_2 } else { output }.get())
                    );
                }
            }
            Self::InstallerTypeSet { index } => {
                debug!("InstallerTypeSet: {index}");
            }
            Self::GetOSInfo => {
                debug!("GetOSInfo");
            }
            Self::ReservedOPCode => {
                debug!("ReservedOPCode");
            }
            Self::LockWindow { on_off } => {
                debug!(
                    "LockWindow: {}",
                    if *on_off == I32::ZERO { "on" } else { "off" }
                );
            }
            Self::FileWriteUTF16LE {
                handle,
                string,
                int_or_string,
            } => {
                debug!("FileWriteUTF16LE: {handle} {string} {int_or_string}");
            }
            Self::FileReadUTF16LE {
                handle,
                output,
                max_length,
                get_char_gets,
            } => {
                debug!("FileReadUTF16LE: {handle} {output} {max_length} {get_char_gets}");
            }
            Self::Log { set, text } => {
                // https://github.com/mcmilk/7-Zip/blob/HEAD/CPP/7zip/Archive/Nsis/NsisIn.cpp#L4799
                if *set == I32::ZERO {
                    debug!("LogText: {}", state.get_string(text.get()));
                } else {
                    debug!("LogSet {}", if *text == I32::ZERO { "off" } else { "on" });
                }
            }
            Self::FindProcess => {
                debug!("FindProcess");
            }
            Self::GetFontVersion => {
                debug!("GetFontVersion");
            }
            Self::GetFontName => {
                debug!("GetFontName");
            }
        }

        Ok(0)
    }
}
