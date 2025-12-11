mod creation_disposition;
mod del_flags;
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

use chrono::DateTime;
use compact_str::{ToCompactString, format_compact};
use creation_disposition::CreationDisposition;
pub use del_flags::DelFlags;
pub use exec_flag::{ExecFlag, ExecFlags};
use generic_access_rights::GenericAccessRights;
use nt_time::FileTime;
use push_pop::PushPop;
use seek_from::SeekFrom;
use show_window::ShowWindow;
use thiserror::Error;
use tracing::debug;
use window_message::WindowMessage;
use zerocopy::{I32, Immutable, KnownLayout, LE, TryFromBytes, U16, U64, transmute};

use super::{file_system::RelativeLocation, registry::RegType, state::NsisState};
use crate::analysis::installers::utils::registry::RegRoot;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum EntryError {
    #[error("Reached aborting entry: {status}")]
    Abort { status: String },
    #[error("Reached invalid entry")]
    Invalid,
    #[error("Execution error")]
    Execute,
    #[error("Ran into infinite loop")]
    InfiniteLoop,
}

#[expect(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
pub enum Entry {
    Invalid = 0u32.to_le(),
    Return = 1u32.to_le(),
    Jump {
        address: I32<LE>,
    } = 2u32.to_le(),
    Abort {
        status: I32<LE>,
    } = 3u32.to_le(),
    Quit = 4u32.to_le(),
    Call {
        address: I32<LE>, // Encoded as +1
    } = 5u32.to_le(),
    UpdateText {
        update_str: I32<LE>,
        ui_st_update_flag: I32<LE>,
    } = 6u32.to_le(),
    Sleep {
        time_ms: I32<LE>,
    } = 7u32.to_le(),
    BringToFront = 8u32.to_le(),
    ChDetailsView {
        list_action: I32<LE>,
        button_action: I32<LE>,
    } = 9u32.to_le(),
    SetFileAttributes {
        filename: I32<LE>,
        attributes: I32<LE>,
    } = 10u32.to_le(),
    CreateDir {
        path: I32<LE>,
        update_install_dir: I32<LE>,
        restrict_ac_1: I32<LE>,
    } = 11u32.to_le(),
    IfFileExists {
        file_name: I32<LE>,
        jump_amount_if_exists: I32<LE>,
        jump_amount_if_not_exists: I32<LE>,
    } = 12u32.to_le(),
    SetFlag {
        r#type: ExecFlag,
        data: I32<LE>,
        mode: I32<LE>,
        restore_control: I32<LE>,
    } = 13u32.to_le(),
    IfFlag {
        on: I32<LE>,
        off: I32<LE>,
        r#type: ExecFlag,
        new_value_mask: I32<LE>,
    } = 14u32.to_le(),
    GetFlag {
        output: I32<LE>,
        r#type: ExecFlag,
    } = 15u32.to_le(),
    Rename {
        old: I32<LE>,
        new: I32<LE>,
        reboot_ok: I32<LE>,
    } = 16u32.to_le(),
    GetFullPathname {
        output: I32<LE>,
        input: I32<LE>,
        long_or_short_file_name: I32<LE>,
    } = 17u32.to_le(),
    SearchPath {
        output: I32<LE>,
        filename: I32<LE>,
    } = 18u32.to_le(),
    GetTempFilename {
        output: I32<LE>,
        base_dir: I32<LE>,
    } = 19u32.to_le(),
    ExtractFile {
        overwrite_flag: I32<LE>,
        name: I32<LE>,
        position: I32<LE>,
        datetime: U64<LE>,
        allow_ignore: I32<LE>,
    } = 20u32.to_le(),
    DeleteFile {
        filename: I32<LE>,
        flags: DelFlags,
    } = 21u32.to_le(),
    MessageBox {
        mb_flags: I32<LE>,
        text: I32<LE>,
    } = 22u32.to_le(),
    RemoveDir {
        path: I32<LE>,
        flags: DelFlags,
    } = 23u32.to_le(),
    StrLen {
        output: I32<LE>,
        input: I32<LE>,
    } = 24u32.to_le(),
    AssignVar {
        variable: I32<LE>,
        string_offset: I32<LE>,
        max_length: I32<LE>,
        start_position: I32<LE>,
    } = 25u32.to_le(),
    StrCmp {
        str_1: I32<LE>,
        str_2: I32<LE>,
        jump_if_equal: I32<LE>,
        jump_if_not_equal: I32<LE>,
        case_sensitive: I32<LE>,
    } = 26u32.to_le(),
    ReadEnv {
        output: I32<LE>,
        string_with_env_variables: I32<LE>,
        is_read: I32<LE>,
    } = 27u32.to_le(),
    IntCmp {
        val_1: I32<LE>,
        val_2: I32<LE>,
        equal: I32<LE>,
        val1_lt_val2: I32<LE>,
        val1_gt_val2: I32<LE>,
        flags: I32<LE>,
    } = 28u32.to_le(),
    IntOp {
        output: I32<LE>,
        input1: I32<LE>,
        input2: I32<LE>,
        operation: I32<LE>,
    } = 29u32.to_le(),
    IntFmt {
        output: I32<LE>,
        format: I32<LE>,
        input: I32<LE>,
        is_64_bit: I32<LE>,
    } = 30u32.to_le(),
    PushPop {
        variable_or_string: I32<LE>,
        push_pop: PushPop,
        exchange: I32<LE>,
    } = 31u32.to_le(),
    FindWindow {
        output: I32<LE>,
        window_class: I32<LE>,
        window_name: I32<LE>,
        window_parent: I32<LE>,
        window_after: I32<LE>,
    } = 32u32.to_le(),
    SendMessage {
        output: I32<LE>,
        handle: I32<LE>,
        msg: I32<LE>,
        wide_param: I32<LE>,
        long_param: I32<LE>,
    } = 33u32.to_le(),
    IsWindow {
        window_handle: I32<LE>,
        jump_if_window: I32<LE>,
        jump_if_not_window: I32<LE>,
    } = 34u32.to_le(),
    GetDialogItem {
        output: I32<LE>,
        dialog: I32<LE>,
        item_id: I32<LE>,
    } = 35u32.to_le(),
    SetCtlColors {
        window_handle: I32<LE>,
        pointer_to_struct_colors: I32<LE>,
    } = 36u32.to_le(),
    SetBrandingImage {
        control: I32<LE>,
        image_type: I32<LE>,
        lr_flags: I32<LE>,
        image_id: I32<LE>,
        output: I32<LE>,
    } = 37u32.to_le(),
    CreateFont {
        handle_output: I32<LE>,
        face_name: I32<LE>,
        height: I32<LE>,
        weight: I32<LE>,
        flags: I32<LE>,
    } = 38u32.to_le(),
    ShowWindow {
        window_handle: I32<LE>,
        show_state: I32<LE>,
        hide_window: I32<LE>,
        enable_window: I32<LE>,
    } = 39u32.to_le(),
    ShellExec {
        see_mask_flag: I32<LE>,
        verb: I32<LE>,
        file: I32<LE>,
        parameters: I32<LE>,
        show_window: I32<LE>,
        status_text: I32<LE>,
    } = 40u32.to_le(),
    Execute {
        complete_command_line: I32<LE>,
        wait_flag: I32<LE>,
        output_error_code: I32<LE>,
    } = 41u32.to_le(),
    GetFileTime {
        file: I32<LE>,
        high_out: I32<LE>,
        low_out: I32<LE>,
    } = 42u32.to_le(),
    GetDLLVersion {
        file: I32<LE>,
        high_out: I32<LE>,
        low_out: I32<LE>,
        fixed_offset: I32<LE>,
    } = 43u32.to_le(),
    RegisterDLL {
        dll_file_name: I32<LE>,
        function_str_ptr: I32<LE>,
        display_text: I32<LE>,
        no_unload: I32<LE>,
    } = 44u32.to_le(),
    CreateShortcut {
        link_file: I32<LE>,
        target_file: I32<LE>,
        parameters: I32<LE>,
        icon_file: I32<LE>,
        create_shortcut: I32<LE>,
    } = 45u32.to_le(),
    CopyFiles {
        source_mask: I32<LE>,
        destination_location: I32<LE>,
        flags: I32<LE>,
        status_text: I32<LE>,
    } = 46u32.to_le(),
    Reboot(I32<LE>) = 47u32.to_le(),
    WriteIni {
        section: I32<LE>,
        name: I32<LE>,
        value: I32<LE>,
        ini_file: I32<LE>,
    } = 48u32.to_le(),
    ReadIni {
        output: I32<LE>,
        section: I32<LE>,
        name: I32<LE>,
        ini_file: I32<LE>,
    } = 49u32.to_le(),
    DeleteReg {
        reserved: I32<LE>,
        root: RegRoot,
        key_name: I32<LE>,
        value_name: I32<LE>,
        flags: I32<LE>,
    } = 50u32.to_le(),
    WriteReg {
        root: RegRoot,
        key_name: I32<LE>,
        value_name: I32<LE>,
        value: I32<LE>,
        r#type: RegType,
        sub_type: RegType,
    } = 51u32.to_le(),
    ReadReg {
        output: I32<LE>,
        root: RegRoot,
        key_name: I32<LE>,
        value_name: I32<LE>,
        r#type: I32<LE>, // DWORD if == 1 else Str
    } = 52u32.to_le(),
    RegEnumKey {
        output: I32<LE>,
        root_key: RegRoot,
        key_name: I32<LE>,
        index: I32<LE>,
        value: I32<LE>,
    } = 53u32.to_le(),
    FileClose {
        handle: I32<LE>,
    } = 54u32.to_le(),
    FileOpen {
        name: I32<LE>,
        open_mode: GenericAccessRights,
        create_mode: CreationDisposition,
        output_handle: I32<LE>,
    } = 55u32.to_le(),
    FileWrite {
        handle: I32<LE>,
        string: I32<LE>,
        int_or_string: I32<LE>,
    } = 56u32.to_le(),
    FileRead {
        handle: I32<LE>,
        output: I32<LE>,
        max_length: I32<LE>,
        get_char_gets: I32<LE>,
    } = 57u32.to_le(),
    FileSeek {
        handle: I32<LE>,
        offset: I32<LE>,
        mode: I32<LE>,
        seek_from: SeekFrom,
    } = 58u32.to_le(),
    FindClose {
        handle: I32<LE>,
    } = 59u32.to_le(),
    FindNext {
        output: I32<LE>,
        handle: I32<LE>,
    } = 60u32.to_le(),
    FindFirst {
        file_spec: I32<LE>,
        output: I32<LE>,
        handle_output: I32<LE>,
    } = 61u32.to_le(),
    WriteUninstaller {
        name: I32<LE>,
        offset: I32<LE>,
        icon_size: I32<LE>,
        alternative_path: I32<LE>,
    } = 62u32.to_le(),
    SectionSet {
        index: I32<LE>,
        output: I32<LE>,
        r#type: I32<LE>,
        call_section_set_flags: I32<LE>,
        output_2: I32<LE>,
    } = 63u32.to_le(),
    InstallerTypeSet {
        index: I32<LE>,
    } = 64u32.to_le(),
    GetOSInfo = 65u32.to_le(),
    ReservedOPCode = 66u32.to_le(),
    LockWindow {
        on_off: I32<LE>,
    } = 67u32.to_le(),
    FileWriteUTF16LE {
        handle: I32<LE>,
        string: I32<LE>,
        int_or_string: I32<LE>,
    } = 68u32.to_le(),
    FileReadUTF16LE {
        handle: I32<LE>,
        output: I32<LE>,
        max_length: I32<LE>,
        get_char_gets: I32<LE>,
    } = 69u32.to_le(),
    Log {
        set: I32<LE>,
        text: I32<LE>,
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
                return Err(EntryError::Invalid);
            }
            Self::Return => {
                debug!("Return");
            }
            Self::Jump { address } => {
                debug!("Jump: {address}");
                return Ok(address.get());
            }
            Self::Abort { status } => {
                let status = state.get_string(status.get());
                debug!(r#"Aborting: "{status}""#);
                return Err(EntryError::Abort {
                    status: status.into_owned(),
                });
            }
            Self::Quit => {
                debug!("Quit");
            }
            Self::Call { address } => {
                let resolved_address = state.resolve_address(address.get()) - 1;
                debug!("Call: {resolved_address}");
                state.execute_code_segment(resolved_address)?;
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

                return if state.file_system.file_exists(&*file_name) {
                    debug!(
                        r#"IfFileExists: jumping to {if_exists} as "{file_name}" exists"#,
                        if_exists = state.resolve_address(jump_amount_if_exists.get()),
                    );
                    Ok(jump_amount_if_exists.get())
                } else {
                    debug!(
                        r#"IfFileExists: jumping to {if_not_exists} as "{file_name}" does NOT exist"#,
                        if_not_exists = state.resolve_address(jump_amount_if_not_exists.get()),
                    );
                    Ok(jump_amount_if_not_exists.get())
                };
            }
            Self::SetFlag {
                r#type,
                data,
                mode,
                restore_control,
            } => {
                // https://github.com/mcmilk/7-Zip/blob/HEAD/CPP/7zip/Archive/Nsis/NsisIn.cpp#L3898

                let value = state.get_int(data.get());
                debug!(
                    "Set{type}Flag {}",
                    match r#type {
                        ExecFlag::AutoClose | ExecFlag::Reboot => {
                            if value == 0 { "false" } else { "true" }
                        }
                        ExecFlag::ShellVarContext => {
                            if value == 0 { "current" } else { "all" }
                        }
                        ExecFlag::Silent => {
                            if value == 0 { "normal" } else { "silent" }
                        }
                        ExecFlag::RegView => {
                            match value {
                                0 => "32",
                                256 => "64",
                                _ => "",
                            }
                        }
                        ExecFlag::DetailsPrint => {
                            match value {
                                0 => "both",
                                2 => "textonly",
                                4 => "listonly",
                                6 => "none",
                                _ => "",
                            }
                        }
                        _ => "",
                    }
                );

                // https://github.com/NSIS-Dev/nsis/blob/v311/Source/exehead/exec.c#L321
                if *mode <= I32::ZERO {
                    if *mode < I32::ZERO {
                        state.status_up_hack = state.exec_flags[*r#type];
                    } else {
                        state.last_used_exec_flags[*r#type] = state.exec_flags[*r#type];
                    }
                    state.exec_flags[*r#type].set(value);
                } else {
                    state.exec_flags[*r#type] = state.last_used_exec_flags[*r#type];
                    if *restore_control < I32::ZERO {
                        state.exec_flags[*r#type] = state.status_up_hack;
                    }
                }
            }
            Self::IfFlag {
                on,
                off,
                r#type,
                new_value_mask,
            } => {
                // https://github.com/NSIS-Dev/nsis/blob/v311/Source/exehead/exec.c#L342
                let exec_flag = &mut state.exec_flags[*r#type];

                let result = if *exec_flag == I32::ZERO {
                    debug!("If{type}Flag: on -> {on}");
                    on
                } else {
                    debug!("If{type}Flag: off -> {off}");
                    off
                };

                *exec_flag &= *new_value_mask;
                return Ok(result.get());
            }
            Self::GetFlag { output, r#type } => {
                debug!("GetFlag: {type}");
                state
                    .variables
                    .insert(output.get() as usize, state.exec_flags[*r#type].to_string());
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
                        .unwrap_or_default(),
                    state.get_string(base_dir.get())
                );
            }
            Self::ExtractFile {
                overwrite_flag: _overwrite_flag,
                name,
                position,
                datetime,
                allow_ignore: _allow_ignore,
            } => {
                let name = state.get_string(name.get());
                let date = if *datetime == U64::MAX_VALUE {
                    debug!(r#"ExtractFile: "{name}"#);
                    None
                } else {
                    let date = DateTime::from(FileTime::new(datetime.get()));
                    debug!(r#"ExtractFile: "{name}" {date}"#);
                    Some(date)
                };
                state
                    .file_system
                    .create_file(&*name, date, position.get().unsigned_abs());
            }
            Self::DeleteFile { filename, flags } => {
                let filename = state.get_string(filename.get());
                debug!(r#"Delete: "{filename}""#);
                state.file_system.delete(filename, *flags);
            }
            Self::MessageBox { mb_flags, text } => {
                let text = state.get_string(text.get());
                debug!(r#"MessageBox: {}, "{text}""#, mb_flags);
            }
            Self::RemoveDir { path, flags } => {
                let path = state.get_string(path.get());
                if path.as_ref() == "" {
                    debug!(r#"RMDir: "" (ignored empty path)"#);
                } else {
                    debug!(r#"RMDir: "{path}""#);
                    state.file_system.delete(path, *flags);
                }
            }
            Self::StrLen { output, input } => {
                let input = state.get_string(input.get());
                debug!(
                    r#"StrLen: "{input}".len() = {} (inserted into {})"#,
                    input.len(),
                    output.get()
                );
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
                let [low, high]: [U16<LE>; 2] = transmute!(*max_length);
                let new_length = if high == U16::ZERO {
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
                    .insert(output.get().unsigned_abs() as usize, formatted);
            }
            Self::PushPop {
                variable_or_string,
                push_pop,
                exchange,
            } => {
                // https://github.com/NSIS-Dev/nsis/blob/v311/Source/exehead/exec.c#L774

                if *exchange != I32::ZERO {
                    let count = exchange.get() as usize;

                    if count < state.stack.len() {
                        let top = state.stack.len() - 1;
                        let target = top - count;
                        state.stack.swap(top, target);
                    } else {
                        return Err(EntryError::Execute);
                    }

                    if count == 1 {
                        debug!("Exchange");
                    } else {
                        debug!("Exchange: {exchange}");
                    }
                } else if push_pop.is_pop() {
                    if let Some(variable) = state.stack.pop() {
                        debug!(r#"Pop: "{variable}""#);
                        state
                            .variables
                            .insert(variable_or_string.get().unsigned_abs() as usize, variable);
                    }
                } else if push_pop.is_push() {
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
            Self::SetBrandingImage {
                control: _control,
                image_type: _image_type,
                lr_flags: _lr_flags,
                image_id,
                output: _output,
            } => {
                debug!("LoadAndSetImage /IMGID={image_id}");
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
                // https://github.com/NSIS-Dev/nsis/blob/v311/Source/exehead/exec.c#L892
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
                // https://github.com/NSIS-Dev/nsis/blob/v311/Source/exehead/fileform.h#L562
                const NO_WORKING_DIRECTORY: I32<LE> = I32::new(0x8000);

                // https://github.com/NSIS-Dev/nsis/blob/v311/Source/exehead/exec.c#L1087
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
                // https://github.com/NSIS-Dev/nsis/blob/v311/Source/exehead/exec.c#L1152
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
            Self::Reboot(bad_food) => {
                // <https://github.com/NSIS-Dev/nsis/blob/v311/Source/exehead/exec.c#L1193>

                const BAD_FOOD: I32<LE> = I32::new(0x0bad_f00d);

                debug!("Reboot");

                if *bad_food != BAD_FOOD {
                    return Err(EntryError::Execute);
                }

                state.exec_flags[ExecFlag::Reboot] += 1;
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
                // https://github.com/NSIS-Dev/nsis/blob/v311/Source/exehead/exec.c#L1240
                // https://github.com/mcmilk/7-Zip/blob/HEAD/CPP/7zip/Archive/Nsis/NsisIn.cpp#L4558

                let key_name = state.get_string(key_name.get());

                if *flags == I32::ZERO {
                    let value_name = state.get_string(value_name.get());

                    debug!(r#"DeleteRegValue: "{root}\{key_name}" "{value_name}""#);

                    state
                        .registry
                        .remove_value_name(*root, &*key_name, &*value_name);
                } else {
                    debug!(
                        r#"DeleteRegKey: {}"{root}\{key_name}""#,
                        if flags.get() & 2 != 0 {
                            "/ifempty "
                        } else {
                            ""
                        }
                    );

                    state.registry.remove_key(*root, &*key_name);
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
                // https://github.com/NSIS-Dev/nsis/blob/v311/Source/exehead/exec.c#L1265

                let key_name = state.get_string(key_name.get());
                let value_name = state.get_string(value_name.get());

                if r#type.is_string() {
                    let value = state.get_string(value.get());

                    if sub_type.is_string() {
                        debug!(r#"WriteRegStr: "{root}\{key_name}" "{value_name}"="{value}""#);
                    } else {
                        debug!(
                            r#"WriteRegExpandStr: "{root}\{key_name}" "{value_name}"="{value}""#
                        );
                    }

                    state
                        .registry
                        .insert_value(*root, key_name, value_name, value);
                } else if r#type.is_dword() {
                    let value = state.get_int(value.get());

                    debug!(r#"WriteRegDWORD: "{root}\{key_name}" "{value_name}"="{value}""#);

                    state.registry.insert_value(
                        *root,
                        key_name,
                        value_name,
                        value.to_compact_string(),
                    );
                } else if r#type.is_binary() {
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
                icon_size: _icon_size,
                alternative_path,
            } => {
                // https://github.com/NSIS-Dev/nsis/blob/v311/Source/exehead/exec.c#L1573

                // https://github.com/NSIS-Dev/nsis/blob/v311/Source/exehead/util.c#L346
                fn is_path_absolute<T: AsRef<[u8]>>(path: T) -> bool {
                    let path = path.as_ref();
                    if let Some((&first, &second)) = path.first().zip(path.get(1)) {
                        (first == b'\\' && second == b'\\')
                            || (first.is_ascii_alphabetic() && second == b':')
                    } else {
                        false
                    }
                }

                let name = state.get_string(name.get());
                let name = if is_path_absolute(&*name) {
                    name
                } else {
                    state.get_string(alternative_path.get())
                };
                debug!(r#"WriteUninstaller: "{name}""#);
                state
                    .file_system
                    .create_file(&*name, None, offset.get().unsigned_abs());
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

    /// Returns `true` if this entry is a return.
    #[expect(unused)]
    #[inline]
    pub const fn is_return(&self) -> bool {
        matches!(self, Self::Return)
    }

    /// Returns `true` if this is a quit entry.
    #[expect(unused)]
    #[inline]
    pub const fn is_quit(&self) -> bool {
        matches!(self, Self::Quit)
    }
}

#[cfg(test)]
mod tests {
    use super::Entry;

    #[test]
    fn entry_size() {
        assert_eq!(size_of::<Entry>(), size_of::<u32>() * 7)
    }

    #[test]
    fn entry_alignment() {
        assert_eq!(align_of::<Entry>(), align_of::<u32>())
    }
}
