use bitflags::bitflags;
use derive_more::{Deref, Display};
use memchr::{memchr, memmem};
use std::cmp::Ordering;

bitflags! {
    #[derive(Debug, Default, PartialEq, Eq)]
    pub struct VersionFlags: u8 {
        const UNICODE = 1 << 0;
        const ISX = 1 << 1;
    }
}

#[derive(Debug, Default, Display, PartialEq, Eq, PartialOrd)]
#[display("{_0}.{_1}.{_2}")]
pub struct InnoVersion(pub u8, pub u8, pub u8, pub u8);

impl PartialEq<(u8, u8, u8)> for InnoVersion {
    fn eq(&self, &(n1, n2, n3): &(u8, u8, u8)) -> bool {
        self.eq(&Self(n1, n2, n3, 0))
    }
}

impl PartialEq<(u8, u8, u8, u8)> for InnoVersion {
    fn eq(&self, &(n1, n2, n3, n4): &(u8, u8, u8, u8)) -> bool {
        self.eq(&Self(n1, n2, n3, n4))
    }
}

impl PartialOrd<(u8, u8, u8)> for InnoVersion {
    fn partial_cmp(&self, &(n1, n2, n3): &(u8, u8, u8)) -> Option<Ordering> {
        self.partial_cmp(&Self(n1, n2, n3, 0))
    }
}

impl PartialOrd<(u8, u8, u8, u8)> for InnoVersion {
    fn partial_cmp(&self, &(n1, n2, n3, n4): &(u8, u8, u8, u8)) -> Option<Ordering> {
        self.partial_cmp(&Self(n1, n2, n3, n4))
    }
}

#[derive(Debug, Default, Deref, Display, PartialEq, Eq)]
#[display("{version}")]
pub struct KnownVersion {
    #[deref]
    pub version: InnoVersion,
    pub variant: VersionFlags,
}

impl PartialEq<(u8, u8, u8)> for KnownVersion {
    fn eq(&self, other: &(u8, u8, u8)) -> bool {
        self.version.eq(other)
    }
}

impl PartialEq<(u8, u8, u8, u8)> for KnownVersion {
    fn eq(&self, other: &(u8, u8, u8, u8)) -> bool {
        self.version.eq(other)
    }
}

impl PartialEq<InnoVersion> for KnownVersion {
    fn eq(&self, other: &InnoVersion) -> bool {
        self.version.eq(other)
    }
}

impl PartialOrd<(u8, u8, u8)> for KnownVersion {
    fn partial_cmp(&self, other: &(u8, u8, u8)) -> Option<Ordering> {
        self.version.partial_cmp(other)
    }
}

impl PartialOrd<(u8, u8, u8, u8)> for KnownVersion {
    fn partial_cmp(&self, other: &(u8, u8, u8, u8)) -> Option<Ordering> {
        self.version.partial_cmp(other)
    }
}

impl PartialOrd<InnoVersion> for KnownVersion {
    fn partial_cmp(&self, other: &InnoVersion) -> Option<Ordering> {
        self.version.partial_cmp(other)
    }
}

impl KnownVersion {
    pub fn from_version_bytes(data: &[u8]) -> Option<Self> {
        const ISX: &[u8; 3] = b"ISX";
        const INNO_SETUP_EXTENSIONS: &[u8; 21] = b"Inno Setup Extensions";

        // Find the first '(' and ')'
        let start_index = memchr(b'(', data)?;
        let end_index = memchr(b')', &data[start_index..])? + start_index;

        // Extract the version bytes within the parentheses
        let version_bytes = &data[start_index + 1..end_index];

        // Split the version string into its components
        let mut parts = version_bytes
            .split(|&b| b == b'.')
            .filter_map(|s| std::str::from_utf8(s).ok()?.parse::<u8>().ok());

        let inno_version = InnoVersion(
            parts.next()?,
            parts.next()?,
            parts.next()?,
            parts.next().unwrap_or_default(),
        );

        // Inno Setup 6.3.0 and above is always only Unicode
        if inno_version >= (6, 3, 0) {
            return Some(Self {
                version: inno_version,
                variant: VersionFlags::UNICODE,
            });
        }

        let mut flags = VersionFlags::empty();

        let remaining_data = &data[end_index..];

        // Check for a Unicode flag within parentheses
        if let Some(u_start_index) = memchr(b'(', remaining_data) {
            if let Some(u_end_index) = memchr(b')', &remaining_data[u_start_index..]) {
                let unicode_flag = &remaining_data[u_start_index + 1..u_start_index + u_end_index];
                if unicode_flag.eq_ignore_ascii_case(b"u") {
                    flags |= VersionFlags::UNICODE;
                }
            }
        }

        if memmem::find(remaining_data, ISX).is_some()
            || memmem::find(remaining_data, INNO_SETUP_EXTENSIONS).is_some()
        {
            flags |= VersionFlags::ISX;
        }

        Some(Self {
            version: inno_version,
            variant: flags,
        })
    }

    pub const fn is_unicode(&self) -> bool {
        self.variant.contains(VersionFlags::UNICODE)
    }

    pub const fn is_isx(&self) -> bool {
        self.variant.contains(VersionFlags::ISX)
    }

    pub fn is_blackbox(&self) -> bool {
        const BLACKBOX_VERSIONS: [InnoVersion; 3] = [
            InnoVersion(5, 3, 10, 0),
            InnoVersion(5, 4, 2, 0),
            InnoVersion(5, 5, 0, 0),
        ];

        self.is_unicode() && BLACKBOX_VERSIONS.contains(&self.version)
    }
}

#[cfg(test)]
mod tests {
    use crate::installers::inno::version::{InnoVersion, KnownVersion, VersionFlags};
    use rstest::rstest;

    #[rstest]
    #[case(b"", InnoVersion(0, 0, 0, 0), VersionFlags::empty())]
    #[case(
        b"Inno Setup Setup Data (1.3.3)",
        InnoVersion(1, 3, 3, 0),
        VersionFlags::empty()
    )]
    #[case(
        b"Inno Setup Setup Data (1.3.12) with ISX (1.3.12.1)",
        InnoVersion(1, 3, 12, 0),
        VersionFlags::ISX
    )]
    #[case(
        b"Inno Setup Setup Data (3.0.3) with ISX (3.0.0)",
        InnoVersion(3, 0, 3, 0),
        VersionFlags::ISX
    )]
    #[case(
        b"My Inno Setup Extensions Setup Data (3.0.4)",
        InnoVersion(3, 0, 4, 0),
        VersionFlags::empty()
    )]
    #[case(
        b"My Inno Setup Extensions Setup Data (3.0.6.1)",
        InnoVersion(3, 0, 6, 1),
        VersionFlags::empty()
    )]
    #[case(
        b"Inno Setup Setup Data (5.3.10)",
        InnoVersion(5, 3, 10, 0),
        VersionFlags::empty()
    )]
    #[case(
        b"Inno Setup Setup Data (5.3.10) (u)",
        InnoVersion(5, 3, 10, 0),
        VersionFlags::UNICODE
    )]
    #[case(
        b"Inno Setup Setup Data (5.5.7) (U)",
        InnoVersion(5, 5, 7, 0),
        VersionFlags::UNICODE
    )]
    #[case(
        b"Inno Setup Setup Data (5.6.0)",
        InnoVersion(5, 6, 0, 0),
        VersionFlags::empty()
    )]
    #[case(
        b"Inno Setup Setup Data (5.6.0) (u)",
        InnoVersion(5, 6, 0, 0),
        VersionFlags::UNICODE
    )]
    #[case(
        b"Inno Setup Setup Data (6.1.0) (u)",
        InnoVersion(6, 1, 0, 0),
        VersionFlags::UNICODE
    )]
    #[case(
        b"Inno Setup Setup Data (6.2.0) (u)",
        InnoVersion(6, 2, 0, 0),
        VersionFlags::UNICODE
    )]
    #[case(
        b"Inno Setup Setup Data (6.3.0)",
        InnoVersion(6, 3, 0, 0),
        VersionFlags::UNICODE
    )]
    #[case(
        b"Inno Setup Setup Data (6.4.0.1)",
        InnoVersion(6, 4, 0, 1),
        VersionFlags::UNICODE
    )]
    fn test_inno_versions(
        #[case] input: &[u8],
        #[case] expected_inno_version: InnoVersion,
        #[case] expected_variant: VersionFlags,
    ) {
        let actual = KnownVersion::from_version_bytes(input).unwrap_or_default();
        assert_eq!(actual.version, expected_inno_version);
        assert_eq!(actual.variant, expected_variant)
    }
}
