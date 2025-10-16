use std::fmt;

use zerocopy::{Immutable, KnownLayout, TryFromBytes};

/// <https://github.com/NSIS-Dev/nsis/blob/v311/Source/exehead/fileform.h#L227>
#[expect(dead_code)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, TryFromBytes, Immutable, KnownLayout)]
#[repr(u128)]
pub enum NsisSignature {
    /// Present in NSIS 1.0..<1.1e
    DeadBeefNsisinst = u128::from_le_bytes(*b"\xEF\xBE\xAD\xDEnsisinstall\0"),
    /// Present in NSIS 1.1e..<1.30
    DeadBeedNullSoftInst = u128::from_le_bytes(*b"\xED\xBE\xAD\xDENullSoftInst"),
    #[default]
    DeadBeefNullsoftInst = u128::from_le_bytes(*b"\xEF\xBE\xAD\xDENullsoftInst"),
}

impl NsisSignature {
    /// Returns the memory representation of this signature as a byte array in little-endian byte
    /// order.
    #[inline]
    pub const fn to_le_bytes(self) -> [u8; size_of::<Self>()] {
        (self as u128).to_le_bytes()
    }
}

impl fmt::Display for NsisSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.to_le_bytes();
        let (magic, signature) = bytes.split_first_chunk().unwrap_or_else(|| unreachable!());

        write!(f, "{:X}", u32::from_le_bytes(*magic))?;
        write!(
            f,
            "{}",
            str::from_utf8(signature).unwrap_or_else(|_| unreachable!())
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use zerocopy::{LE, U32, ValidityError, transmute, try_transmute};

    use super::NsisSignature;

    #[test]
    fn size() {
        assert_eq!(size_of::<NsisSignature>(), size_of::<u128>());
    }

    #[rstest]
    #[case::deadbeef_nsisinstall(
        transmute!([
            U32::<LE>::new(0xDEADBEEF),
            U32::<LE>::new(0x7369736E),
            U32::<LE>::new(0x74736E69),
            U32::<LE>::new(0x006C6C61)
        ]),
        "DEADBEEFnsisinstall\0"
    )]
    #[case::deadbeed_nullsoft_inst(
        transmute!([
            U32::<LE>::new(0xDEADBEED),
            U32::<LE>::new(0x6C6C754E),
            U32::<LE>::new(0x74666F53),
            U32::<LE>::new(0x74736E49)
        ]),
        "DEADBEEDNullSoftInst"
    )]
    #[case::deadbeef_nullsoft_inst(
        transmute!([
            U32::<LE>::new(0xDEADBEEF),
            U32::<LE>::new(0x6C6C754E),
            U32::<LE>::new(0x74666F73),
            U32::<LE>::new(0x74736E49)
        ]),
        "DEADBEEFNullsoftInst"
    )]
    fn signature_from_bytes(
        #[case] bytes: [u8; size_of::<NsisSignature>()],
        #[case] expected: &str,
    ) -> Result<(), ValidityError<[u8; size_of::<NsisSignature>()], NsisSignature>> {
        let nsis_signature: NsisSignature = try_transmute!(bytes)?;

        assert_eq!(nsis_signature.to_string(), expected);

        Ok(())
    }
}
