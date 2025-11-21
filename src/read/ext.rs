use std::io::{Read, Result};

use zerocopy::{ByteOrder, FromBytes, U16, U32};

/// Extends [`Read`] with methods for reading numbers. (For `std::io`.)
///
/// Most of the methods defined here have an unconstrained type parameter that
/// must be explicitly instantiated. Typically, it is instantiated with either
/// the [`BigEndian`] or [`LittleEndian`] types defined in this crate.
///
/// [`BigEndian`]: enum.BigEndian.html
/// [`LittleEndian`]: enum.LittleEndian.html
/// [`Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
pub trait ReadBytesExt: Read {
    /// Read a type that implements [`FromBytes`] from the underlying reader.
    ///
    /// # Errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    #[inline]
    fn read_t<T: FromBytes>(&mut self) -> Result<T> {
        T::read_from_io(self)
    }

    /// Reads an unsigned 16-bit integer from the underlying reader.
    ///
    /// # Errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    #[inline]
    fn read_u16<T: ByteOrder>(&mut self) -> Result<u16> {
        U16::<T>::read_from_io(self).map(U16::get)
    }

    /// Reads an unsigned 32-bit integer from the underlying reader.
    ///
    /// # Errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    #[inline]
    fn read_u32<T: ByteOrder>(&mut self) -> Result<u32> {
        U32::<T>::read_from_io(self).map(U32::get)
    }
}

/// All types that implement `Read` get methods defined in `ReadBytesExt` for free.
impl<R: Read + ?Sized> ReadBytesExt for R {}
