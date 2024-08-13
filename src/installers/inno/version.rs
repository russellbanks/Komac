use bitflags::bitflags;
use std::cmp::Ordering;

bitflags! {
    #[derive(PartialEq, Eq)]
    pub struct VersionFlags: u32 {
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

impl PartialEq<InnoVersion> for KnownVersion {
    fn eq(&self, other: &InnoVersion) -> bool {
        self.version == *other
    }
}

impl PartialOrd<InnoVersion> for KnownVersion {
    fn partial_cmp(&self, other: &InnoVersion) -> Option<Ordering> {
        self.version.partial_cmp(other)
    }
}

impl KnownVersion {
    pub fn is_isx(&self) -> bool {
        self.variant == VersionFlags::ISX
    }

    pub fn is_unicode(&self) -> bool {
        self.variant == VersionFlags::UNICODE
    }
}

#[derive(PartialEq, Eq, PartialOrd)]
pub struct InnoVersion(pub u32, pub u32, pub u32);

pub const KNOWN_VERSIONS: [KnownVersion; 12] = [
    KnownVersion {
        name: "Inno Setup Setup Data (5.5.7)", // Ambiguous
        version: InnoVersion(5, 5, 7),
        variant: VersionFlags::empty(),
    },
    KnownVersion {
        name: "Inno Setup Setup Data (5.5.7) (u)", // Ambiguous
        version: InnoVersion(5, 5, 7),
        variant: VersionFlags::UNICODE,
    },
    KnownVersion {
        name: "Inno Setup Setup Data (5.5.7) (U)", // Ambiguous
        version: InnoVersion(5, 5, 7),
        variant: VersionFlags::UNICODE,
    },
    KnownVersion {
        name: "Inno Setup Setup Data (5.5.8) (u)", // Unofficial
        version: InnoVersion(5, 5, 8),
        variant: VersionFlags::UNICODE,
    },
    KnownVersion {
        name: "Inno Setup Setup Data (5.6.0)",
        version: InnoVersion(5, 6, 0),
        variant: VersionFlags::empty(),
    },
    KnownVersion {
        name: "Inno Setup Setup Data (5.6.0) (u)",
        version: InnoVersion(5, 6, 0),
        variant: VersionFlags::UNICODE,
    },
    KnownVersion {
        name: "Inno Setup Setup Data (5.6.2)",
        version: InnoVersion(5, 6, 2),
        variant: VersionFlags::empty(),
    },
    KnownVersion {
        name: "Inno Setup Setup Data (5.6.2) (u)",
        version: InnoVersion(5, 6, 2),
        variant: VersionFlags::UNICODE,
    },
    KnownVersion {
        name: "Inno Setup Setup Data (6.0.0) (u)",
        version: InnoVersion(6, 0, 0),
        variant: VersionFlags::UNICODE,
    },
    KnownVersion {
        name: "Inno Setup Setup Data (6.1.0) (u)",
        version: InnoVersion(6, 1, 0),
        variant: VersionFlags::UNICODE,
    },
    KnownVersion {
        name: "Inno Setup Setup Data (6.2.0) (u)",
        version: InnoVersion(6, 2, 0),
        variant: VersionFlags::UNICODE,
    },
    KnownVersion {
        name: "Inno Setup Setup Data (6.3.0)",
        version: InnoVersion(6, 3, 0),
        variant: VersionFlags::empty(),
    },
];
