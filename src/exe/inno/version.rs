use bitflags::bitflags;

bitflags! {
    struct VersionFlags: u32 {
        const BITS_16 = 1 << 0;
        const UNICODE = 1 << 1;
        const ISX = 1 << 2;
    }
}

pub struct KnownVersion {
    pub name: &'static str,
    pub version: InnoVersion,
    pub variant: VersionFlags,
}

#[derive(PartialEq, PartialOrd)]
pub struct InnoVersion(pub u32, pub u32, pub u32, pub u32);

pub const KNOWN_VERSIONS: [KnownVersion; 12] = [
    KnownVersion {
        name: "Inno Setup Setup Data (5.5.7)", // Ambiguous
        version: InnoVersion(5, 5, 7, 0),
        variant: VersionFlags::empty(),
    },
    KnownVersion {
        name: "Inno Setup Setup Data (5.5.7) (u)", // Ambiguous
        version: InnoVersion(5, 5, 7, 0),
        variant: VersionFlags::UNICODE,
    },
    KnownVersion {
        name: "Inno Setup Setup Data (5.5.7) (U)", // Ambiguous
        version: InnoVersion(5, 5, 7, 0),
        variant: VersionFlags::UNICODE,
    },
    KnownVersion {
        name: "Inno Setup Setup Data (5.5.8) (u)", // Unofficial
        version: InnoVersion(5, 5, 8, 0),
        variant: VersionFlags::UNICODE,
    },
    KnownVersion {
        name: "Inno Setup Setup Data (5.6.0)",
        version: InnoVersion(5, 6, 0, 0),
        variant: VersionFlags::empty(),
    },
    KnownVersion {
        name: "Inno Setup Setup Data (5.6.0) (u)",
        version: InnoVersion(5, 6, 0, 0),
        variant: VersionFlags::UNICODE,
    },
    KnownVersion {
        name: "Inno Setup Setup Data (5.6.2)",
        version: InnoVersion(5, 6, 2, 0),
        variant: VersionFlags::empty(),
    },
    KnownVersion {
        name: "Inno Setup Setup Data (5.6.2) (u)",
        version: InnoVersion(5, 6, 2, 0),
        variant: VersionFlags::UNICODE,
    },
    KnownVersion {
        name: "Inno Setup Setup Data (6.0.0) (u)",
        version: InnoVersion(6, 0, 0, 0),
        variant: VersionFlags::UNICODE,
    },
    KnownVersion {
        name: "Inno Setup Setup Data (6.1.0) (u)",
        version: InnoVersion(6, 1, 0, 0),
        variant: VersionFlags::UNICODE,
    },
    KnownVersion {
        name: "Inno Setup Setup Data (6.2.0) (u)",
        version: InnoVersion(6, 2, 0, 0),
        variant: VersionFlags::UNICODE,
    },
    KnownVersion {
        name: "Inno Setup Setup Data (6.3.0)",
        version: InnoVersion(6, 3, 0, 0),
        variant: VersionFlags::empty(),
    },
];
