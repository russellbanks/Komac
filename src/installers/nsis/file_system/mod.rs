mod directory;
mod file;
mod item;

use std::{iter::FilterMap, slice::Iter};

use camino::{Utf8Component, Utf8Path};
use chrono::{DateTime, Utc};
pub use directory::Directory;
pub use file::File;
use indextree::{Arena, Node, NodeId};
use item::Item;

#[derive(Debug)]
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
        let root = arena.new_node(Item::Directory(Directory::new_root()));
        Self {
            arena,
            root,
            current_dir: root,
        }
    }

    pub fn create_directory<T>(&mut self, name: T, location: RelativeLocation) -> NodeId
    where
        T: AsRef<Utf8Path>,
    {
        const INST_DIR: &str = "$INSTDIR";
        const OUT_DIR: &str = "$_OUTDIR";

        let current = match location {
            RelativeLocation::Root => self.root,
            RelativeLocation::Current => self.current_dir,
        };

        let mut current = current;

        for mut component in name.as_ref().components() {
            if [INST_DIR, OUT_DIR].contains(&component.as_str()) {
                component = Utf8Component::RootDir;
            }

            match component {
                Utf8Component::ParentDir => {
                    if let Some(parent) = self.arena.get(current).and_then(Node::parent) {
                        current = parent;
                    }
                }
                Utf8Component::RootDir => {
                    while let Some(parent) = self.arena.get(current).and_then(Node::parent) {
                        current = parent;
                    }
                }
                Utf8Component::Normal(part) => {
                    if let Some(directory) = current.children(&self.arena).find(|&id| {
                        self.arena
                            .get(id)
                            .map(Node::get)
                            .is_some_and(|item| item.name() == part)
                    }) {
                        current = directory;
                    } else {
                        let new_dir = Directory::new(part);
                        current = current.append_value(new_dir.into(), &mut self.arena);
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

    #[inline]
    pub fn create_file<T, D>(&mut self, name: T, modified: D) -> NodeId
    where
        T: AsRef<Utf8Path>,
        D: Into<DateTime<Utc>>,
    {
        let path = name.as_ref();
        let mut components = path
            .components()
            .filter_map(|component| {
                if let Utf8Component::Normal(part) = component {
                    Some(part)
                } else {
                    None
                }
            })
            .peekable();

        let mut current = self.current_dir;
        while let Some(directory) = components.next() {
            if components.peek().is_none() {
                let file = File::new(directory, modified);
                let id = current.append_value(file.into(), &mut self.arena);
                return id;
            }
            current = self.create_directory(directory, RelativeLocation::Current);
        }

        panic!()
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

    pub fn files(&self) -> impl Iterator<Item = &File> {
        self.root
            .descendants(&self.arena)
            .filter_map(|id| self.arena.get(id).and_then(|node| node.get().as_file()))
    }

    pub fn directories(&self) -> impl Iterator<Item = &Directory> {
        self.root.descendants(&self.arena).filter_map(|id| {
            self.arena
                .get(id)
                .and_then(|node| node.get().as_directory())
        })
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
