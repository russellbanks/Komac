use std::fmt::{Display, Formatter};
use std::ops::Rem;

pub struct Ordinal<T>(pub T);

impl<T> Ordinal<T>
where
    T: Rem<Output = T> + From<u8> + PartialEq + Copy,
{
    pub fn suffix(&self) -> &str {
        match self.0 % T::from(100) {
            n if n == T::from(11 | 12 | 13) => "th",
            _ => match self.0 % T::from(10) {
                n if n == T::from(1) => "st",
                n if n == T::from(2) => "nd",
                n if n == T::from(3) => "rd",
                _ => "th",
            },
        }
    }
}

impl<T> Display for Ordinal<T>
where
    T: Display + Rem<Output = T> + From<u8> + PartialEq + Copy,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.0, self.suffix())
    }
}
