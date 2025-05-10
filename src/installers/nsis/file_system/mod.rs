mod item;

use std::{
    fmt,
    iter::{FilterMap, once},
    slice::Iter,
};

use camino::{Utf8Component, Utf8Path};
use chrono::{DateTime, Utc};
use indextree::{Arena, Node, NodeId};
pub use item::Item;
use itertools::{Either, Itertools, Position};

use super::strings::PredefinedVar;

pub struct FileSystem {
    arena: Arena<Item>,
    root: NodeId,
    current_dir: NodeId,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RelativeLocation {
    Root,
    Current,
}

impl FileSystem {
    pub fn new() -> Self {
        let mut arena = Arena::new();
        let root = arena.new_node(Item::new_root());
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

    pub fn create_directory<T>(&mut self, name: T, location: RelativeLocation) -> NodeId
    where
        T: AsRef<Utf8Path>,
    {
        let current = match location {
            RelativeLocation::Root => self.root,
            RelativeLocation::Current => self.current_dir,
        };

        let mut current = current;

        for component in Self::parse_path(&name) {
            match component {
                Utf8Component::ParentDir => {
                    if let Some(parent) = self.arena.get(current).and_then(Node::parent) {
                        current = parent;
                    }
                }
                Utf8Component::RootDir => current = self.root,
                Utf8Component::Normal(part) => {
                    if let Some(directory) = current.children(&self.arena).find(|&id| {
                        self.arena
                            .get(id)
                            .is_some_and(|item| item.get().name() == part)
                    }) {
                        current = directory;
                    } else {
                        current = current.append_value(Item::new_directory(part), &mut self.arena);
                    }
                }
                _ => {}
            }
        }

        current
    }

    pub fn set_directory<T>(&mut self, path: T, location: RelativeLocation)
    where
        T: AsRef<Utf8Path>,
    {
        self.current_dir = self.create_directory(path, location);
    }

    pub fn create_file<T, D>(&mut self, path: T, created_at: D) -> Option<NodeId>
    where
        T: AsRef<Utf8Path>,
        D: Into<Option<DateTime<Utc>>>,
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
            let file = Item::new_file(file_name, created_at);
            Some(directory.append_value(file, &mut self.arena))
        }
    }

    pub fn file_exists<T>(&self, name: T) -> bool
    where
        T: AsRef<Utf8Path>,
    {
        let mut current = self.current_dir;

        for (position, component) in Self::parse_path(&name).with_position() {
            match component {
                Utf8Component::ParentDir => {
                    if let Some(parent) = self.arena.get(current).and_then(Node::parent) {
                        current = parent;
                    }
                }
                Utf8Component::RootDir => current = self.root,
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

    pub fn delete_file<T: ?Sized>(&mut self, name: &T) -> bool
    where
        T: AsRef<str>,
    {
        if let Some(file) = self.current_dir.children(&self.arena).find(|&id| {
            self.arena
                .get(id)
                .map(Node::get)
                .is_some_and(|item| item.is_file() && item.name() == name.as_ref())
        }) {
            file.remove(&mut self.arena);
            return true;
        }

        false
    }

    pub fn directories(&self) -> impl Iterator<Item = &Item> {
        self.root
            .descendants(&self.arena)
            .filter_map(|id| self.arena.get(id).map(Node::get))
            .filter(|item| item.is_directory())
    }

    pub fn files(&self) -> impl Iterator<Item = &Item> {
        self.root
            .descendants(&self.arena)
            .filter_map(|id| self.arena.get(id).map(Node::get))
            .filter(|item| item.is_file())
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Item> {
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
    type Item = &'a Item;

    type IntoIter = FilterMap<Iter<'a, Node<Item>>, fn(&'a Node<Item>) -> Option<&'a Item>>;

    fn into_iter(self) -> Self::IntoIter {
        self.arena
            .iter()
            .filter_map(|node| (!node.is_removed()).then_some(node.get()))
    }
}
