use crate::types::architecture::Architecture;
use bitflags::bitflags;

bitflags! {
    /// Used before Inno Setup 6.3 where the architecture was stored in a single byte
    #[derive(Debug, Default)]
    pub struct StoredArchitecture: u8 {
        const ARCHITECTURE_UNKNOWN = 1 << 0;
        const X86 = 1 << 1;
        const AMD64 = 1 << 2;
        const IA64 = 1 << 3;
        const ARM64 = 1 << 4;
    }
}

impl StoredArchitecture {
    pub fn to_identifiers(&self) -> ArchitectureIdentifiers {
        let mut identifiers = ArchitectureIdentifiers::empty();
        match self {
            flags if flags.contains(Self::AMD64) || flags.contains(Self::IA64) => {
                identifiers |= ArchitectureIdentifiers::X64_OS;
            }
            flags if flags.contains(Self::ARM64) => identifiers |= ArchitectureIdentifiers::ARM64,
            flags if flags.contains(Self::X86) => identifiers |= ArchitectureIdentifiers::X86_OS,
            _ => {}
        }
        identifiers
    }
}

bitflags! {
    /// <https://jrsoftware.org/ishelp/index.php?topic=archidentifiers>
    #[derive(Debug, Default)]
    pub struct ArchitectureIdentifiers: u8 {
        /// Matches systems capable of running 32-bit Arm binaries. Only Arm64 Windows includes such
        /// support.
        const ARM32_COMPATIBLE = 1 << 0;
        /// Matches systems running Arm64 Windows.
        const ARM64 = 1 << 1;
        /// Matches systems running 64-bit Windows, regardless of OS architecture.
        const WIN64 = 1 << 2;
        /// Matches systems capable of running x64 binaries. This includes systems running x64
        /// Windows, and also Arm64-based Windows 11 systems, which have the ability to run x64
        /// binaries via emulation.
        const X64_COMPATIBLE = 1 << 3;
        /// Matches systems running x64 Windows only — not any other systems that have the ability
        /// to run x64 binaries via emulation.
        const X64_OS = 1 << 4;
        /// Matches systems capable of running 32-bit x86 binaries. This includes systems running
        /// x86 Windows, x64 Windows, and also Arm64 Windows 10 and 11 systems, which have the
        /// ability to run x86 binaries via emulation.
        const X86_COMPATIBLE = 1 << 5;
        /// Matches systems running 32-bit x86 Windows only.
        const X86_OS = 1 << 6;
    }
}

impl ArchitectureIdentifiers {
    pub fn from_spaced_list(s: &str) -> Self {
        const ARM32_COMPATIBLE: &str = "arm32compatible";
        const ARM64: &str = "arm64";
        const WIN64: &str = "win64";
        const X64_COMPATIBLE: &str = "x64compatible";
        const X64_OS: &str = "x64os";
        /// Before Inno Setup 6.3, x64os was named x64. The compiler still accepts x64 as an alias
        /// for x64os, but will emit a deprecation warning when used.
        const X64: &str = "x64";
        const X86_COMPATIBLE: &str = "x86compatible";
        const X86_OS: &str = "x86os";
        /// Before Inno Setup 6.3, x86os was named x86. The compiler still accepts x86 as an alias
        /// for x86os.
        const X86: &str = "x86";

        let mut architecture_flags = Self::empty();
        for architecture in s.split(' ') {
            match architecture {
                ARM32_COMPATIBLE => architecture_flags |= Self::ARM32_COMPATIBLE,
                ARM64 => architecture_flags |= Self::ARM64,
                WIN64 => architecture_flags |= Self::WIN64,
                X64_COMPATIBLE => architecture_flags |= Self::X64_COMPATIBLE,
                X64_OS | X64 => architecture_flags |= Self::X64_OS,
                X86_COMPATIBLE => architecture_flags |= Self::X86_COMPATIBLE,
                X86_OS | X86 => architecture_flags |= Self::X86_OS,
                _ => {}
            }
        }
        if architecture_flags.is_empty() {
            architecture_flags |= Self::X86_COMPATIBLE;
        }
        architecture_flags
    }

    pub const fn to_winget_architecture(&self) -> Option<Architecture> {
        if self.contains(Self::X64_OS)
            || self.contains(Self::WIN64)
            || self.contains(Self::X64_COMPATIBLE)
        {
            Some(Architecture::X64)
        } else if self.contains(Self::ARM64) || self.contains(Self::ARM32_COMPATIBLE) {
            Some(Architecture::Arm64)
        } else if self.contains(Self::X86_OS) || self.contains(Self::X86_COMPATIBLE) {
            Some(Architecture::X86)
        } else {
            None
        }
    }
}
