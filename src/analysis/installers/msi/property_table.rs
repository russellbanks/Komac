use std::{
    borrow::Borrow,
    hash::Hash,
    io,
    io::{Read, Seek},
    iter::Map,
};

use compact_str::CompactString;
use indexmap::{IndexMap, map::Iter};
use msi::{Package, Select};
use tracing::debug;

use super::PROPERTY;

/// Represents the `Property` table from a Windows Installer (MSI) database.
///
/// This table stores key-value pairs where the key is the property name and the value is the
/// corresponding property value. It provides convenient methods to load, query, and iterate over
/// properties defined in the MSI.
///
/// For more details, see the [Microsoft documentation on the Property Table].
///
/// [Microsoft documentation on the Property Table]: https://learn.microsoft.com/windows/win32/msi/property-table
#[derive(Clone, Debug)]
pub struct PropertyTable(IndexMap<CompactString, CompactString>);

impl PropertyTable {
    /// Creates a new [`PropertyTable`] by reading all properties from the given MSI package.
    ///
    /// This function queries the `Property` table in the MSI database and collects all
    /// `(Property, Value)` pairs into a [`PropertyTable`].
    pub fn new<R: Read + Seek>(msi: &mut Package<R>) -> io::Result<Self> {
        const VALUE: &str = "Value";

        Ok(msi
            .select_rows(Select::table(PROPERTY))?
            .filter_map(|row| {
                row[PROPERTY]
                    .as_str()
                    .map(CompactString::from)
                    .zip(row[VALUE].as_str().map(CompactString::from))
            })
            .inspect(|(property, value)| debug!(%property, %value))
            .collect())
    }

    /// Returns a reference to the value corresponding to the given property name.
    ///
    /// The key may be any borrowed form of the map's key type, but [`Hash`] and [`Eq`] on the
    /// borrowed form *must* match those for the key type.
    pub fn get<Q>(&self, property: &Q) -> Option<&str>
    where
        CompactString: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.0.get(property).map(CompactString::as_str)
    }

    /// An iterator visiting all property-value pairs in their order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.into_iter()
    }
}

impl<A, B> FromIterator<(A, B)> for PropertyTable
where
    A: Into<CompactString>,
    B: Into<CompactString>,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (A, B)>,
    {
        Self(
            iter.into_iter()
                .map(|(property, value)| (property.into(), value.into()))
                .collect(),
        )
    }
}

impl<'a> IntoIterator for &'a PropertyTable {
    type Item = (&'a str, &'a str);

    type IntoIter = Map<
        Iter<'a, CompactString, CompactString>,
        fn((&'a CompactString, &'a CompactString)) -> (&'a str, &'a str),
    >;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0
            .iter()
            .map(|(property, value)| (property.as_str(), value.as_str()))
    }
}
