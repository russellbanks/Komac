use zerocopy::{LE, U16};

/// A single UTF-16 code unit, stored little-endian as it is in the NSIS strings block.
pub type WideChar = U16<LE>;
