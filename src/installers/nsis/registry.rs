use std::{borrow::Cow, collections::HashMap};

use zerocopy::{Immutable, KnownLayout, TryFromBytes};

use crate::installers::utils::registry::RegRoot;

type Values<'data> = HashMap<Cow<'data, str>, Cow<'data, str>>;

type Keys<'data> = HashMap<Cow<'data, str>, Values<'data>>;

// Registry root -< Key name -< Value name - Value
#[derive(Debug)]
pub struct Registry<'data>(HashMap<RegRoot, Keys<'data>>);

const CURRENT_VERSION_UNINSTALL: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall";

impl<'data> Registry<'data> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get_product_code(&self) -> Option<&str> {
        // Find the first Software\Microsoft\Windows\CurrentVersion\Uninstall\{PRODUCT_CODE} key
        // under any root and extract the product code from it
        self.0.values().find_map(|keys| {
            keys.keys().find_map(|key| {
                key.rsplit_once('\\').and_then(|(parent, product_code)| {
                    (parent == CURRENT_VERSION_UNINSTALL).then_some(product_code)
                })
            })
        })
    }

    pub fn set_value<K, N, V>(&mut self, root: RegRoot, key: K, name: N, value: V)
    where
        K: Into<Cow<'data, str>>,
        N: Into<Cow<'data, str>>,
        V: Into<Cow<'data, str>>,
    {
        self.0
            .entry(root)
            .or_default()
            .entry(key.into())
            .or_default()
            .insert(name.into(), value.into());
    }

    pub fn remove_value(&mut self, name: &str) -> Option<Cow<'data, str>> {
        self.0
            .values_mut()
            .find_map(|keys| keys.values_mut().find_map(|values| values.remove(name)))
    }
}

/// <https://github.com/kichik/nsis/blob/HEAD/Source/Platform.h#L672>
#[expect(dead_code)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, TryFromBytes, KnownLayout, Immutable)]
#[repr(i32)]
pub enum RegType {
    #[default]
    None = 0i32.to_le(),
    String = 1i32.to_le(),
    ExpandedString = 2i32.to_le(),
    Binary = 3i32.to_le(),
    DWord = 4i32.to_le(),
    MultiString = 5i32.to_le(),
}
