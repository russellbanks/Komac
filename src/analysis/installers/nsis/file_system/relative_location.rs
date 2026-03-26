use std::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RelativeLocation {
    Root,
    Current,
}

impl fmt::Display for RelativeLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
