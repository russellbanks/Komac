use crate::installers::nsis::file_system::{directory::Directory, file::File};

#[derive(Clone, Debug)]
pub enum Item {
    File(File),
    Directory(Directory),
}

impl Item {
    pub fn name(&self) -> &str {
        match self {
            Self::Directory(directory) => directory.name(),
            Self::File(file) => file.name(),
        }
    }

    #[inline]
    pub const fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }

    #[inline]
    pub const fn is_directory(&self) -> bool {
        matches!(self, Self::Directory(_))
    }

    pub const fn as_file(&self) -> Option<&File> {
        match self {
            Self::Directory(_) => None,
            Self::File(file) => Some(file),
        }
    }

    pub const fn as_directory(&self) -> Option<&Directory> {
        match self {
            Self::Directory(directory) => Some(directory),
            Self::File(_) => None,
        }
    }
}

impl From<File> for Item {
    #[inline]
    fn from(file: File) -> Self {
        Self::File(file)
    }
}

impl From<Directory> for Item {
    #[inline]
    fn from(directory: Directory) -> Self {
        Self::Directory(directory)
    }
}
