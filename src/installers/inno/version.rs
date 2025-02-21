use std::cmp::Ordering;

use bitflags::bitflags;
use derive_more::Display;
use memchr::{memchr, memmem};

bitflags! {
    #[derive(Debug, Default, PartialEq, Eq)]
    pub struct VersionFlags: u8 {
        const UNICODE = 1 << 0;
        const ISX = 1 << 1;
    }
}

#[derive(Debug, Default, Display, Eq)]
#[display("{major}.{minor}.{patch}.{revision}")]
pub struct InnoVersion {
    major: u8,
    minor: u8,
    patch: u8,
    revision: u8,
    variant: VersionFlags,
}

impl InnoVersion {
    pub const fn new(major: u8, minor: u8, patch: u8, revision: u8) -> Self {
        Self {
            major,
            minor,
            patch,
            revision,
            variant: VersionFlags::empty(),
        }
    }

    #[cfg(test)]
    pub const fn new_with_variant(
        major: u8,
        minor: u8,
        patch: u8,
        revision: u8,
        variant: VersionFlags,
    ) -> Self {
        Self {
            major,
            minor,
            patch,
            revision,
            variant,
        }
    }

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

        let inno_version = Self::new(
            parts.next()?,
            parts.next()?,
            parts.next()?,
            parts.next().unwrap_or_default(),
        );

        // Inno Setup 6.3.0 and above is always only Unicode
        if inno_version >= (6, 3, 0) {
            return Some(Self {
                variant: VersionFlags::UNICODE,
                ..inno_version
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
            variant: flags,
            ..inno_version
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
            InnoVersion::new(5, 3, 10, 0),
            InnoVersion::new(5, 4, 2, 0),
            InnoVersion::new(5, 5, 0, 0),
        ];

        self.is_unicode() && BLACKBOX_VERSIONS.contains(self)
    }
}

impl PartialEq for InnoVersion {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major
            && self.minor == other.minor
            && self.patch == other.patch
            && self.revision == other.revision
    }
}

impl PartialEq<(u8, u8, u8, u8)> for InnoVersion {
    fn eq(&self, &(n1, n2, n3, n4): &(u8, u8, u8, u8)) -> bool {
        self == &Self::new(n1, n2, n3, n4)
    }
}

impl PartialEq<(u8, u8, u8)> for InnoVersion {
    fn eq(&self, &(n1, n2, n3): &(u8, u8, u8)) -> bool {
        self == &(n1, n2, n3, 0)
    }
}

impl PartialOrd for InnoVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for InnoVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then_with(|| self.minor.cmp(&other.minor))
            .then_with(|| self.patch.cmp(&other.patch))
            .then_with(|| self.revision.cmp(&other.revision))
    }
}

impl PartialOrd<(u8, u8, u8)> for InnoVersion {
    fn partial_cmp(&self, &(n1, n2, n3): &(u8, u8, u8)) -> Option<Ordering> {
        self.partial_cmp(&Self::new(n1, n2, n3, 0))
    }
}

impl PartialOrd<(u8, u8, u8, u8)> for InnoVersion {
    fn partial_cmp(&self, &(n1, n2, n3, n4): &(u8, u8, u8, u8)) -> Option<Ordering> {
        self.partial_cmp(&Self::new(n1, n2, n3, n4))
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use rstest::rstest;

    use crate::installers::inno::version::{InnoVersion, VersionFlags};

    #[rstest]
    #[case(b"", InnoVersion::new(0, 0, 0, 0))]
    #[case(b"Inno Setup Setup Data (1.3.3)", InnoVersion::new(1, 3, 3, 0))]
    #[case(
        b"Inno Setup Setup Data (1.3.12) with ISX (1.3.12.1)",
        InnoVersion::new_with_variant(1, 3, 12, 0, VersionFlags::ISX)
    )]
    #[case(
        b"Inno Setup Setup Data (3.0.3) with ISX (3.0.0)",
        InnoVersion::new_with_variant(3, 0, 3, 0, VersionFlags::ISX)
    )]
    #[case(
        b"My Inno Setup Extensions Setup Data (3.0.4)",
        InnoVersion::new(3, 0, 4, 0)
    )]
    #[case(
        b"My Inno Setup Extensions Setup Data (3.0.6.1)",
        InnoVersion::new(3, 0, 6, 1)
    )]
    #[case(b"Inno Setup Setup Data (5.3.10)", InnoVersion::new(5, 3, 10, 0))]
    #[case(
        b"Inno Setup Setup Data (5.3.10) (u)",
        InnoVersion::new_with_variant(5, 3, 10, 0, VersionFlags::UNICODE)
    )]
    #[case(
        b"Inno Setup Setup Data (5.5.7) (U)",
        InnoVersion::new_with_variant(5, 5, 7, 0, VersionFlags::UNICODE)
    )]
    #[case(b"Inno Setup Setup Data (5.6.0)", InnoVersion::new(5, 6, 0, 0))]
    #[case(
        b"Inno Setup Setup Data (5.6.0) (u)",
        InnoVersion::new_with_variant(5, 6, 0, 0, VersionFlags::UNICODE)
    )]
    #[case(
        b"Inno Setup Setup Data (6.1.0) (u)",
        InnoVersion::new_with_variant(6, 1, 0, 0, VersionFlags::UNICODE)
    )]
    #[case(
        b"Inno Setup Setup Data (6.2.0) (u)",
        InnoVersion::new_with_variant(6, 2, 0, 0, VersionFlags::UNICODE)
    )]
    #[case(
        b"Inno Setup Setup Data (6.3.0)",
        InnoVersion::new_with_variant(6, 3, 0, 0, VersionFlags::UNICODE)
    )]
    #[case(
        b"Inno Setup Setup Data (6.4.0.1)",
        InnoVersion::new_with_variant(6, 4, 0, 1, VersionFlags::UNICODE)
    )]
    fn inno_version_from_bytes(#[case] input: &[u8], #[case] expected_inno_version: InnoVersion) {
        assert_eq!(
            InnoVersion::from_version_bytes(input).unwrap_or_default(),
            expected_inno_version
        );
    }

    #[test]
    fn inno_version_equality() {
        let version = InnoVersion::new(1, 2, 3, 4);
        let unicode_version = InnoVersion::new_with_variant(1, 2, 3, 4, VersionFlags::UNICODE);
        let isx_version = InnoVersion::new_with_variant(1, 2, 3, 4, VersionFlags::ISX);

        // Check that version flags aren't included in comparison
        assert_eq!(version, unicode_version);
        assert_eq!(version, isx_version);
        assert_eq!(unicode_version, isx_version);

        // Check that comparison equality returns the same as normal equality
        assert_eq!(version.cmp(&unicode_version), Ordering::Equal);
        assert_eq!(version.cmp(&isx_version), Ordering::Equal);
        assert_eq!(unicode_version.cmp(&isx_version), Ordering::Equal);

        // Check equivalent tuples
        assert_eq!(InnoVersion::new(1, 2, 3, 0), (1, 2, 3));
        assert_eq!(version, (1, 2, 3, 4));
    }

    #[test]
    fn inno_version_comparison() {
        let version = InnoVersion::new(1, 2, 3, 4);

        assert!(version < InnoVersion::new(1, 2, 3, 5));
        assert!(version > InnoVersion::new(1, 2, 3, 3));

        assert!(version < (1, 2, 3, 5));
        assert!(version > (1, 2, 3, 3));

        assert!(version > (1, 2, 3));
        assert!(version < (1, 2, 4));
    }
}
