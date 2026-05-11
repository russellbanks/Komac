use std::{fmt::Display, io, ops::Deref};

use thiserror::Error;
use zerocopy::{ConvertError, KnownLayout, SizeError};

use super::EntryError;

#[derive(Error, Debug)]
pub enum NsisError {
    #[error("File is not a NSIS installer")]
    NotNsisFile,
    #[error(transparent)]
    InvalidEntry(#[from] EntryError),
    #[error("{0}")]
    ZeroCopy(String),
    #[error(transparent)]
    Io(#[from] io::Error),
}

impl<A: Display, S: Display, V: Display> From<ConvertError<A, S, V>> for NsisError {
    fn from(err: ConvertError<A, S, V>) -> Self {
        Self::ZeroCopy(err.to_string())
    }
}

impl<Src, Dst: ?Sized> From<SizeError<Src, Dst>> for NsisError
where
    Src: Deref,
    Dst: KnownLayout,
{
    fn from(err: SizeError<Src, Dst>) -> Self {
        Self::ZeroCopy(err.to_string())
    }
}
