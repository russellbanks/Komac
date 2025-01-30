use zerocopy::{Immutable, KnownLayout, TryFromBytes};

#[expect(dead_code)]
#[derive(
    Copy, Clone, Debug, Default, Hash, PartialEq, Eq, TryFromBytes, KnownLayout, Immutable,
)]
#[repr(u32)]
pub enum RegRoot {
    #[default]
    ShellContext = 0u32.to_le(),
    HKeyClassesRoot = 0x8000_0000u32.to_le(),
    HKeyCurrentUser = 0x8000_0001u32.to_le(),
    HKeyLocalMachine = 0x8000_0002u32.to_le(),
    HKeyUsers = 0x8000_0003u32.to_le(),
    HKeyPerformanceData = 0x8000_0004u32.to_le(),
    HKeyCurrentConfig = 0x8000_0005u32.to_le(),
    HKeyDynamicData = 0x8000_0006u32.to_le(),
    HKeyPerformanceText = 0x8000_0050u32.to_le(),
    HKeyPerformanceNLSText = 0x8000_0060u32.to_le(),
}
