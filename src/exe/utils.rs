use num_traits::{PrimInt, Unsigned};
use object::ReadRef;
use std::mem;

pub fn align<T: PrimInt + Unsigned>(value: T, boundary: T) -> T {
    if value % boundary == T::zero() {
        value
    } else {
        value + (boundary - (value % boundary))
    }
}

pub fn get_widestring_size<'data, R: ReadRef<'data>>(data: R, offset: u64) -> u64 {
    let mut index = offset;
    for i in (index..data.len().unwrap()).step_by(mem::size_of::<u16>()) {
        if data.read_at::<u16>(i).unwrap() == &0 {
            index = i;
            break;
        }
    }

    (index - offset) / mem::size_of::<u16>() as u64
}
