use derive_more::Display;
use zerocopy::{Immutable, KnownLayout, TryFromBytes};

#[expect(dead_code)]
#[derive(Copy, Clone, Debug, Display, PartialEq, Eq, TryFromBytes, KnownLayout, Immutable)]
#[repr(i32)]
pub enum ExecFlag {
    AutoClose = 0i32,
    ShellVarContext = 1i32.to_le(),
    Errors = 2i32.to_le(),
    Abort = 3i32.to_le(),
    Reboot = 4i32.to_le(),
    RebootCalled = 5i32.to_le(),
    CurInstType = 6i32.to_le(),
    PluginApiVersion = 7i32.to_le(),
    Silent = 8i32.to_le(),
    InstDirError = 9i32.to_le(),
    RightToLeft = 10i32.to_le(),
    ErrorLevel = 11i32.to_le(),
    RegView = 12i32.to_le(),
    DetailsPrint = 13i32.to_le(),
}
