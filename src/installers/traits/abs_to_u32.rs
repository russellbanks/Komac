pub trait AbsToU32 {
    fn abs_to_u32(self) -> u32;
}

impl AbsToU32 for u32 {
    fn abs_to_u32(self) -> u32 {
        self
    }
}

impl AbsToU32 for i32 {
    fn abs_to_u32(self) -> u32 {
        self.unsigned_abs()
    }
}
