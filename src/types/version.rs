use compact_str::CompactString;
use derive_more::Display;
use itertools::{EitherOrBoth, Itertools};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use smallvec::SmallVec;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

#[derive(Clone, Debug, Default, Display, Eq, SerializeDisplay, DeserializeFromStr)]
#[display("{raw}")]
pub struct Version {
    raw: CompactString,
    parts: SmallVec<[VersionPart; 4]>, // Most versions have 4 parts or fewer
}

impl Version {
    pub const SEPARATOR: char = '.';

    pub fn new(input: &str) -> Self {
        let mut trimmed = input.trim();

        // If there is a digit before the delimiter, or no delimiters, trim off all leading
        // non-digit characters
        if let Some(digit_pos) = trimmed.find(|char: char| char.is_ascii_digit()) {
            if trimmed
                .find('.')
                .is_none_or(|delimiter_pos| digit_pos < delimiter_pos)
            {
                trimmed = &trimmed[digit_pos..];
            }
        }

        let mut parts = trimmed
            .split(Self::SEPARATOR)
            .map(VersionPart::new)
            .collect::<SmallVec<[_; 4]>>();

        if parts.is_empty() {
            parts.push(VersionPart::new(trimmed));
        }

        let droppable_parts = parts
            .iter()
            .rev()
            .take_while(|part| part.is_droppable())
            .count();

        parts.truncate(parts.len() - droppable_parts);

        Self {
            raw: CompactString::from(input),
            parts,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }

    pub fn is_latest(&self) -> bool {
        const LATEST: &str = "latest";

        self.raw.eq_ignore_ascii_case(LATEST)
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.parts.eq(&other.parts)
    }
}

impl Hash for Version {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.parts.hash(state);
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.parts
            .iter()
            .zip_longest(other.parts.iter())
            .map(|pair| match pair {
                EitherOrBoth::Both(a, b) => a.cmp(b),
                EitherOrBoth::Left(a) => a.cmp(&VersionPart::default()),
                EitherOrBoth::Right(b) => VersionPart::default().cmp(b),
            })
            .find(|&ordering| ordering != Ordering::Equal)
            .unwrap_or(Ordering::Equal)
    }
}

impl FromStr for Version {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
struct VersionPart {
    number: u64,
    supplement: Option<CompactString>,
}

impl VersionPart {
    pub fn new(input: &str) -> Self {
        let input = input.trim();

        let split_index = input
            .find(|char: char| !char.is_ascii_digit())
            .unwrap_or(input.len());

        let (number_str, supplement) = input.split_at(split_index);

        Self {
            number: number_str.parse().unwrap_or_default(),
            supplement: Option::from(supplement)
                .filter(|supplement| !supplement.is_empty())
                .map(CompactString::from),
        }
    }

    // WinGet ignores trailing parts that are 0 and have no supplemental value
    pub const fn is_droppable(&self) -> bool {
        self.number == 0 && self.supplement.is_none()
    }
}

impl PartialOrd for VersionPart {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VersionPart {
    fn cmp(&self, other: &Self) -> Ordering {
        self.number.cmp(&other.number).then_with(|| {
            match (self.supplement.as_deref(), other.supplement.as_deref()) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(a), Some(b)) => a.cmp(b),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::types::version::Version;
    use rstest::rstest;
    use std::cmp::Ordering;
    use std::hash::{DefaultHasher, Hash, Hasher};

    #[rstest]
    #[case("1.0", "1.0.0")]
    #[case("1.2.00.3", "1.2.0.3")]
    #[case("1.2.003.4", "1.2.3.4")]
    #[case("01.02.03.04", "1.2.3.4")]
    #[case("1.2.03-beta", "1.2.3-beta")]
    #[case("1.0", "1.0 ")]
    #[case("1.0", "1. 0")]
    #[case("1.0", "1.0.")]
    #[case("1.0", "Version 1.0")]
    #[case("foo1", "bar1")]
    fn version_equality(#[case] left: &str, #[case] right: &str) {
        let left = Version::new(left);
        let right = Version::new(right);
        assert_eq!(left, right);
        assert_eq!(left.cmp(&right), Ordering::Equal);
    }

    #[rstest]
    #[case("1", "2")]
    #[case("1.2-rc", "1.2")]
    #[case("1.0-rc", "1.0")]
    #[case("1.0.0-rc", "1")]
    #[case("22.0.0-rc.1", "22.0.0")]
    #[case("22.0.0-rc.1", "22.0.0.1")]
    #[case("22.0.0-rc.1", "22.0.0.1-rc")]
    #[case("22.0.0-rc.1", "22.0.0-rc.1.1")]
    #[case("22.0.0-rc.1.1", "22.0.0-rc.1.2")]
    #[case("22.0.0-rc.1.2", "22.0.0-rc.2")]
    #[case("v0.0.1", "0.0.2")]
    #[case("v0.0.1", "v0.0.2")]
    #[case("1.a2", "1.b1")]
    #[case("alpha", "beta")]
    fn version_comparison(#[case] left: &str, #[case] right: &str) {
        let left = Version::new(left);
        let right = Version::new(right);
        assert!(left < right);
        assert!(right > left);
    }

    #[rstest]
    #[case("1", "2")]
    #[case("1-rc", "1")]
    #[case("1-a2", "1-b1")]
    #[case("alpha", "beta")]
    fn version_part_comparison(#[case] left: &str, #[case] right: &str) {
        let left = Version::new(left);
        let right = Version::new(right);
        assert!(left < right);
        assert!(right > left);
    }

    #[test]
    fn version_hash() {
        // If two keys are equal, their hashes must also be equal
        // https://doc.rust-lang.org/std/hash/trait.Hash.html#hash-and-eq

        let version1 = Version::new("1.2.3");
        let version2 = Version::new("1.2.3.0");
        assert_eq!(version1, version2);

        let mut version1_hasher = DefaultHasher::default();
        version1.hash(&mut version1_hasher);

        let mut version2_hasher = DefaultHasher::default();
        version2.hash(&mut version2_hasher);

        assert_eq!(version1_hasher.finish(), version2_hasher.finish());
    }

    #[test]
    fn only_supplement() {
        const ALPHA: &str = "alpha";

        let version = Version::new(ALPHA);
        assert_eq!(version.parts.len(), 1);
        assert_eq!(version.parts[0].number, 0);
        assert_eq!(version.parts[0].supplement.as_deref(), Some(ALPHA));
    }
}
