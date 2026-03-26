mod entry;
mod relative_location;

use std::{
    fmt,
    iter::{FilterMap, once},
    slice::Iter,
};

use camino::{Utf8Component, Utf8Path};
use chrono::{DateTime, Utc};
pub use entry::FsEntry;
use indextree::{Arena, Node, NodeId};
use itertools::{Either, Itertools, Position};
pub use relative_location::RelativeLocation;

use super::{entry::DelFlags, strings::PredefinedVar};

pub struct FileSystem {
    arena: Arena<FsEntry>,
    root: NodeId,
    current_dir: NodeId,
}

impl FileSystem {
    /// Creates a new mock filesystem.
    pub fn new() -> Self {
        let mut arena = Arena::new();
        let root = arena.new_node(FsEntry::new_root());
        Self {
            arena,
            root,
            current_dir: root,
        }
    }

    fn parse_path<'path, T>(path: &'path T) -> impl Iterator<Item = Utf8Component<'path>>
    where
        T: AsRef<Utf8Path> + 'path,
    {
        let path = path.as_ref();

        let components = path.components().filter(|component| {
            ![PredefinedVar::InstDir, PredefinedVar::_OutDir]
                .iter()
                .contains(component.as_str())
        });

        if path
            .components()
            .next()
            .is_some_and(|component| PredefinedVar::all().iter().contains(component.as_str()))
        {
            Either::Left(once(Utf8Component::RootDir).chain(components))
        } else {
            Either::Right(components)
        }
    }

    /// Creates a new directory from a path relative to a [location].
    /// The current directory is not changed. To both create and change the current directory, see
    /// [set_directory].
    ///
    /// [location]: RelativeLocation
    /// [set_directory]: Self::set_directory
    pub fn create_directory<T>(&mut self, name: T, location: RelativeLocation) -> NodeId
    where
        T: AsRef<Utf8Path>,
    {
        let mut current = match location {
            RelativeLocation::Root => self.root,
            RelativeLocation::Current => self.current_dir,
        };

        for component in Self::parse_path(&name) {
            match component {
                Utf8Component::RootDir => current = self.root,
                Utf8Component::ParentDir => {
                    if let Some(parent) = current.parent(&self.arena) {
                        current = parent;
                    }
                }
                Utf8Component::Normal(part) => {
                    if let Some(directory) = current.children(&self.arena).find(|&id| {
                        self.arena
                            .get(id)
                            .is_some_and(|item| item.get().name() == part)
                    }) {
                        current = directory;
                    } else {
                        current =
                            current.append_value(FsEntry::new_directory(part), &mut self.arena);
                    }
                }
                _ => {}
            }
        }

        current
    }

    /// Sets the current directory from a path, relative to a [location], creating it and all
    /// necessary parent directories if they do not already exist.
    ///
    /// [location]: RelativeLocation
    pub fn set_directory<T>(&mut self, path: T, location: RelativeLocation)
    where
        T: AsRef<Utf8Path>,
    {
        self.current_dir = self.create_directory(path, location);
    }

    /// Creates a file from a path relative to the current directory, an optional modified at
    /// [datetime], and a position.
    ///
    /// [datetime]: DateTime<Utc>
    pub fn create_file<T, D, P>(&mut self, path: T, modified_at: D, position: P) -> Option<NodeId>
    where
        T: AsRef<Utf8Path>,
        D: Into<Option<DateTime<Utc>>>,
        P: Into<u64>,
    {
        let path = path.as_ref();

        let file_name = path.file_name()?;

        let directory = if let Some(parent) = path.parent() {
            self.create_directory(parent, RelativeLocation::Current)
        } else {
            self.current_dir
        };

        if let Some(file) = directory.children(&self.arena).find(|&id| {
            self.arena
                .get(id)
                .is_some_and(|node| node.get().name() == file_name)
        }) {
            Some(file)
        } else {
            let file = FsEntry::new_file(file_name, modified_at, position);
            Some(directory.append_value(file, &mut self.arena))
        }
    }

    /// Returns true if the given path exists relative to a [location].
    ///
    /// [location]: RelativeLocation
    pub fn exists<T>(&self, path: T, location: RelativeLocation) -> bool
    where
        T: AsRef<Utf8Path>,
    {
        let mut current = match location {
            RelativeLocation::Root => self.root,
            RelativeLocation::Current => self.current_dir,
        };

        for (position, component) in Self::parse_path(&path).with_position() {
            match component {
                Utf8Component::RootDir => current = self.root,
                Utf8Component::ParentDir => {
                    if let Some(parent) = current.parent(&self.arena) {
                        current = parent;
                    }
                }
                Utf8Component::Normal(part) => {
                    if let Some(directory) = current.children(&self.arena).find(|&id| {
                        self.arena
                            .get(id)
                            .is_some_and(|item| item.get().name() == part)
                    }) {
                        if matches!(position, Position::Last | Position::Only) {
                            return true;
                        }
                        current = directory;
                    } else {
                        break;
                    }
                }
                _ => {}
            }
        }

        false
    }

    /// Deletes a path.
    ///
    /// If `flags` contains [`SIMPLE`], the file is deleted relative to the current directory.
    /// If `flags` contains [`DIRECTORY`], the directory is deleted relative to the filesystem root.
    ///
    /// [`SIMPLE`]: DelFlags::SIMPLE
    /// [`DIRECTORY`]: DelFlags::DIRECTORY
    pub fn delete<T>(&mut self, path: T, flags: DelFlags) -> bool
    where
        T: AsRef<str> + Sized,
    {
        let path = path.as_ref();

        // Return false if the path is empty; it cannot be deleted
        if path.is_empty() {
            return false;
        }

        if flags.contains(DelFlags::SIMPLE) {
            if let Some(file) = self.current_dir.children(&self.arena).find(|&id| {
                self.arena
                    .get(id)
                    .map(Node::get)
                    .is_some_and(|item| item.is_file() && item.name() == path)
            }) {
                file.remove(&mut self.arena);
                return true;
            }
        } else if flags.contains(DelFlags::DIRECTORY) {
            let mut current = self.root;

            for component in Self::parse_path(&path) {
                match component {
                    Utf8Component::RootDir => current = self.root,
                    Utf8Component::ParentDir => {
                        if let Some(parent) = current.parent(&self.arena) {
                            current = parent;
                        }
                    }
                    Utf8Component::Normal(part) => {
                        if let Some(child) = current.children(&self.arena).find(|&id| {
                            self.arena
                                .get(id)
                                .is_some_and(|node| node.get().name() == part)
                        }) {
                            current = child;
                        } else {
                            return false;
                        }
                    }
                    _ => {}
                }
            }

            if let Some(node) = self.arena.get(current) {
                let parent_node_id = node.parent();

                let mut removed = false;

                if node.get().is_directory() {
                    removed = if flags.contains(DelFlags::RECURSE) {
                        current.remove_subtree(&mut self.arena);
                        true
                    } else {
                        // Only delete the directory if it's empty
                        if node.first_child().is_none() {
                            current.remove(&mut self.arena);
                            true
                        } else {
                            false
                        }
                    };
                }

                // Delete the parent directory if it:
                //   1. No longer has any children
                //   2. Is a Windows environment variable (e.g. %Temp%)
                if removed
                    && let Some(parent_node_id) = parent_node_id
                    && let Some(parent_node) = self.arena.get(parent_node_id)
                {
                    let parent_name = parent_node.get().name();

                    if parent_node.first_child().is_none()
                        && parent_name.starts_with('%')
                        && parent_name.ends_with('%')
                    {
                        parent_node_id.remove(&mut self.arena);
                    }
                }

                return removed;
            }
        }

        false
    }

    /// Returns an iterator over all entries in the filesystem.
    ///
    /// The iteration order is identical to [`NodeId::descendants`], but with node IDs resolved to
    /// [filesystem entries].
    ///
    /// [filesystem entries]: FsEntry
    pub fn entries(&self) -> impl Iterator<Item = &FsEntry> {
        self.root
            .descendants(&self.arena)
            .filter_map(|id| self.arena.get(id).map(Node::get))
    }

    /// Returns an iterator over all directories in the filesystem.
    ///
    /// The iteration order is identical to [`NodeId::descendants`], but with node IDs resolved to
    /// [filesystem entries].
    ///
    /// [filesystem entries]: FsEntry
    pub fn directories(&self) -> impl Iterator<Item = &FsEntry> {
        self.entries().filter(|item| item.is_directory())
    }

    /// Returns an iterator over all files in the filesystem.
    ///
    /// The iteration order is identical to [`NodeId::descendants`], but with node IDs resolved to
    /// [filesystem entries].
    ///
    /// [filesystem entries]: FsEntry
    pub fn files(&self) -> impl Iterator<Item = &FsEntry> {
        self.entries().filter(|item| item.is_file())
    }

    /// Returns an iterator of all filesystem entries in storage-order.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &FsEntry> {
        self.into_iter()
    }
}

impl fmt::Debug for FileSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.root.debug_pretty_print(&self.arena).fmt(f)
    }
}

impl fmt::Display for FileSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.root.debug_pretty_print(&self.arena).fmt(f)
    }
}

impl Default for FileSystem {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> IntoIterator for &'a FileSystem {
    type Item = &'a FsEntry;

    type IntoIter =
        FilterMap<Iter<'a, Node<FsEntry>>, fn(&'a Node<FsEntry>) -> Option<&'a FsEntry>>;

    /// Returns an iterator of all filesystem entries in storage-order.
    fn into_iter(self) -> Self::IntoIter {
        self.arena
            .iter()
            .filter_map(|node| (!node.is_removed()).then_some(node.get()))
    }
}
