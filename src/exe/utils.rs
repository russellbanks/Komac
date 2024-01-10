pub const fn align(value: usize, boundary: usize) -> usize {
    if value % boundary == 0 {
        value
    } else {
        value + (boundary - (value % boundary))
    }
}
