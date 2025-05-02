use std::{
    borrow::{Borrow, Cow},
    hash::Hash,
};

use indexmap::IndexMap;
use zerocopy::{Immutable, KnownLayout, TryFromBytes};

use crate::installers::utils::registry::RegRoot;

type Key<'data> = Cow<'data, str>;

type ValueName<'data> = Cow<'data, str>;

type Value<'data> = Cow<'data, str>;

type Values<'data> = IndexMap<ValueName<'data>, Value<'data>>;

type Keys<'data> = IndexMap<Key<'data>, Values<'data>>;

/// A mock registry for simulating execution of `WriteReg` and `DeleteReg` entries.
///
/// This represents a hierarchical registry structure modeled as:
///
/// - A **Registry root** maps to multiple **Keys** (one-to-many).
/// - Each **Key** maps to multiple **Value names** (one-to-many).
/// - Each **Value name** maps to a single **Value** (one-to-one).
///
/// Hierarchy:
///
/// ```
/// Registry root ─┬─> Key name ─┬─> Value name ──> Value
///                │             └──────────────> ...
///                └────────────> ...
/// ```
#[derive(Debug)]
pub struct Registry<'data>(IndexMap<RegRoot, Keys<'data>>);

const CURRENT_VERSION_UNINSTALL: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall";

impl<'data> Registry<'data> {
    pub fn new() -> Self {
        Self(IndexMap::new())
    }

    /// Finds the first `Software\Microsoft\Windows\CurrentVersion\Uninstall\{PRODUCT_CODE}` key
    /// under any root and extracts the product code from it.
    pub fn product_code(&self) -> Option<&str> {
        self.0.values().find_map(|keys| {
            keys.keys().find_map(|key| {
                key.rsplit_once('\\').and_then(|(parent, product_code)| {
                    (parent == CURRENT_VERSION_UNINSTALL).then_some(product_code)
                })
            })
        })
    }

    /// Inserts the value into the registry.
    ///
    /// If the registry did not have this value name present, [`None`] is returned.
    ///
    /// If the registry did have this value name present, the value is updated, and the old value is
    /// returned.
    pub fn insert_value<K, N, V>(
        &mut self,
        root: RegRoot,
        key: K,
        name: N,
        value: V,
    ) -> Option<Value<'data>>
    where
        K: Into<Key<'data>>,
        N: Into<ValueName<'data>>,
        V: Into<Value<'data>>,
    {
        self.0
            .entry(root)
            .or_default()
            .entry(key.into())
            .or_default()
            .insert(name.into(), value.into())
    }

    /// Removes the entire set of values associated with a given key from a specific registry root.
    pub fn remove_key<K: ?Sized>(&mut self, root: RegRoot, key: &K) -> Option<Values<'data>>
    where
        Key<'data>: Borrow<K>,
        K: Hash + Eq,
    {
        self.0.get_mut(&root)?.shift_remove(key)
    }

    /// Removes a specific named value from a key within a given registry root.
    pub fn remove_value_name<K: ?Sized, N: ?Sized>(
        &mut self,
        root: RegRoot,
        key: &K,
        name: &N,
    ) -> Option<Value<'data>>
    where
        Key<'data>: Borrow<K>,
        ValueName<'data>: Borrow<N>,
        K: Hash + Eq,
        N: Hash + Eq,
    {
        self.0.get_mut(&root)?.get_mut(key)?.shift_remove(name)
    }

    /// Removes the first occurrence of a value with the specified name across all registry roots
    /// and keys.
    pub fn remove_value_by_name<N: ?Sized>(&mut self, name: &N) -> Option<Value<'data>>
    where
        ValueName<'data>: Borrow<N>,
        N: Hash + Eq,
    {
        self.0.values_mut().find_map(|keys| {
            keys.values_mut()
                .find_map(|values| values.shift_remove(name))
        })
    }
}

/// <https://github.com/kichik/nsis/blob/HEAD/Source/Platform.h#L672>
#[expect(dead_code)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
pub enum RegType {
    #[default]
    None = 0u32.to_le(),
    String = 1u32.to_le(),
    ExpandedString = 2u32.to_le(),
    Binary = 3u32.to_le(),
    DWord = 4u32.to_le(),
    MultiString = 5u32.to_le(),
}
