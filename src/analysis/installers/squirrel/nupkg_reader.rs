use std::{
    fs::File,
    io::{Read, Result, Seek, SeekFrom},
};

use crate::analysis::installers::pe::resource::SectionReader;

pub enum NupkgReader<R: Read + Seek> {
    File(File),
    Section(SectionReader<R>),
}

impl<R: Read + Seek> Read for NupkgReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self {
            Self::File(file) => file.read(buf),
            Self::Section(reader) => reader.read(buf),
        }
    }
}

impl<R: Read + Seek> Seek for NupkgReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match self {
            Self::File(file) => file.seek(pos),
            Self::Section(reader) => reader.seek(pos),
        }
    }
}
