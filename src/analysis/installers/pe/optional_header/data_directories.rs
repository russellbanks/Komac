use std::{io, slice, vec};

use zerocopy::{FromZeros, IntoBytes};

use super::DataDirectory;

#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct DataDirectories(Vec<DataDirectory>);

impl DataDirectories {
    #[inline]
    const fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    fn new_zeroed(len: usize) -> Self
    where
        DataDirectory: Sized,
    {
        Self(vec![DataDirectory::new_zeroed(); len])
    }

    pub fn read_from<R>(mut src: R, len: usize) -> io::Result<Self>
    where
        R: io::Read,
    {
        // Initialise zeroed data directories
        let mut data_directories = Self::new_zeroed(len);

        // Read the data directories
        for data_directory in data_directories.inner_mut() {
            src.read_exact(data_directory.as_mut_bytes())?;
        }

        Ok(data_directories)
    }

    #[inline]
    const fn inner(&self) -> &[DataDirectory] {
        self.0.as_slice()
    }

    #[inline]
    const fn inner_mut(&mut self) -> &mut [DataDirectory] {
        self.0.as_mut_slice()
    }

    #[inline]
    pub const fn export_table(&self) -> Option<&DataDirectory> {
        self.inner().first()
    }

    #[inline]
    pub fn import_table(&self) -> Option<&DataDirectory> {
        self.inner().get(1)
    }

    #[inline]
    pub fn resource_table(&self) -> Option<&DataDirectory> {
        self.inner().get(2)
    }

    #[inline]
    pub fn exception_table(&self) -> Option<&DataDirectory> {
        self.inner().get(3)
    }

    #[inline]
    pub fn certificate_table(&self) -> Option<&DataDirectory> {
        self.inner().get(4)
    }

    #[inline]
    pub fn base_relocation_table(&self) -> Option<&DataDirectory> {
        self.inner().get(5)
    }

    #[inline]
    pub fn debug_table(&self) -> Option<&DataDirectory> {
        self.inner().get(6)
    }

    #[inline]
    pub fn architecture(&self) -> Option<&DataDirectory> {
        self.inner().get(7)
    }

    #[inline]
    pub fn global_ptr(&self) -> Option<&DataDirectory> {
        self.inner().get(8)
    }

    #[inline]
    pub fn tls_table(&self) -> Option<&DataDirectory> {
        self.inner().get(9)
    }

    #[inline]
    pub fn load_config_table(&self) -> Option<&DataDirectory> {
        self.inner().get(10)
    }

    #[inline]
    pub fn bound_import_table(&self) -> Option<&DataDirectory> {
        self.inner().get(11)
    }

    #[inline]
    pub fn import_address_table(&self) -> Option<&DataDirectory> {
        self.inner().get(12)
    }

    #[inline]
    pub fn delay_import_descriptor(&self) -> Option<&DataDirectory> {
        self.inner().get(13)
    }

    #[inline]
    pub fn clr_runtime_header(&self) -> Option<&DataDirectory> {
        self.inner().get(14)
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &DataDirectory> {
        self.into_iter()
    }
}

impl IntoIterator for DataDirectories {
    type Item = DataDirectory;

    type IntoIter = vec::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a DataDirectories {
    type Item = &'a DataDirectory;

    type IntoIter = slice::Iter<'a, DataDirectory>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
