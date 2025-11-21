use super::{ImageResourceDataEntry, ResourceDirectoryTable};

/// Data associated with a resource directory entry.
#[derive(Debug)]
pub enum ResourceDirectoryEntryData {
    /// A subtable entry.
    Table(ResourceDirectoryTable),

    /// A resource data entry.
    Data(ImageResourceDataEntry),
}

impl ResourceDirectoryEntryData {
    /// Converts to an option of table.
    ///
    /// Helper for iterator filtering.
    pub fn table(self) -> Option<ResourceDirectoryTable> {
        match self {
            Self::Table(dir) => Some(dir),
            Self::Data(_) => None,
        }
    }

    /// Converts to an option of data entry.
    ///
    /// Helper for iterator filtering.
    pub fn data(self) -> Option<ImageResourceDataEntry> {
        match self {
            Self::Data(rsc) => Some(rsc),
            Self::Table(_) => None,
        }
    }
}
