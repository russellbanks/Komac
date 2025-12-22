use std::{
    collections::VecDeque,
    io::{Read, Seek},
};

use super::{
    IdImageResourceDirectoryEntry, IdOrName, IdOrNamedImageResourceDirectoryEntry, Resource,
    ResourceDirectory, ResourceDirectoryEntryData,
};

pub struct ResourceIter<R: Read + Seek> {
    dir: ResourceDirectory<R>,

    type_entries: VecDeque<IdOrNamedImageResourceDirectoryEntry>,
    name_entries: VecDeque<IdImageResourceDirectoryEntry>,
    lang_entries: VecDeque<IdImageResourceDirectoryEntry>,

    current_type: IdOrName,
    current_name_id: u32,
}

impl<R: Read + Seek> ResourceIter<R> {
    pub fn new(mut dir: ResourceDirectory<R>) -> Self {
        let root = dir.current_directory_table();

        Self {
            type_entries: root.entries().collect(),
            dir,

            name_entries: VecDeque::new(),

            lang_entries: VecDeque::new(),

            current_type: IdOrName::default(),
            current_name_id: 0,
        }
    }
}

impl<R: Read + Seek> Iterator for ResourceIter<R> {
    type Item = Resource;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Language level
            if let Some(lang_entry) = self.lang_entries.pop_front() {
                return Some(Resource {
                    r#type: self.current_type.clone(),
                    name_id: self.current_name_id,
                    language_id: lang_entry.id(),
                    entry: lang_entry.data(&mut self.dir).ok()?.data()?,
                });
            }

            // Name level
            if let Some(name_entry) = self.name_entries.pop_front() {
                self.lang_entries = name_entry
                    .data(&mut self.dir)
                    .ok()
                    .and_then(ResourceDirectoryEntryData::table)?
                    .id_entries()
                    .collect();
                self.current_name_id = name_entry.id();

                continue;
            }

            // Type level
            if let Some(type_entry) = self.type_entries.pop_front() {
                self.current_type = type_entry.id_or_name();

                self.name_entries = type_entry
                    .data(&mut self.dir)
                    .ok()
                    .and_then(ResourceDirectoryEntryData::table)?
                    .id_entries()
                    .collect();

                continue;
            }

            return None;
        }
    }
}
