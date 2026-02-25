use std::{borrow::Borrow, hash::Hash};

use indexmap::IndexMap;
use zerocopy::{Immutable, KnownLayout, TryFromBytes};

use crate::analysis::installers::utils::registry::RegRoot;

type Key = String;

type ValueName = String;

pub type Value = String;

type Values = IndexMap<ValueName, Value>;

type Keys = IndexMap<Key, Values>;

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
#[derive(Clone, Debug)]
pub struct Registry(IndexMap<RegRoot, Keys>);

const CURRENT_VERSION_UNINSTALL: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall";

impl Registry {
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
    ) -> Option<Value>
    where
        K: Into<Key>,
        N: Into<ValueName>,
        V: Into<Value>,
    {
        self.0
            .entry(root)
            .or_default()
            .entry(key.into())
            .or_default()
            .insert(name.into(), value.into())
    }

    /// Removes the entire set of values associated with a given key from a specific registry root.
    pub fn remove_key<K>(&mut self, root: RegRoot, key: &K) -> Option<Values>
    where
        Key: Borrow<K>,
        K: Hash + Eq + ?Sized,
    {
        self.0.get_mut(&root)?.shift_remove(key)
    }

    /// Removes a specific named value from a key within a given registry root.
    pub fn remove_value_name<K, N>(&mut self, root: RegRoot, key: &K, name: &N) -> Option<Value>
    where
        Key: Borrow<K>,
        ValueName: Borrow<N>,
        K: Hash + Eq + ?Sized,
        N: Hash + Eq + ?Sized,
    {
        self.0.get_mut(&root)?.get_mut(key)?.shift_remove(name)
    }

    /// Returns a reference to the first occurrence of a value with the specified name across all
    /// registry roots and keys.
    pub fn get_value_by_name<N>(&self, name: &N) -> Option<&Value>
    where
        ValueName: Borrow<N>,
        N: Hash + Eq + ?Sized,
    {
        self.0
            .values()
            .find_map(|keys| keys.values().find_map(|values| values.get(name)))
    }

    /// Removes the first occurrence of a value with the specified name across all registry roots
    /// and keys.
    #[expect(unused)]
    pub fn remove_value_by_name<N>(&mut self, name: &N) -> Option<Value>
    where
        ValueName: Borrow<N>,
        N: Hash + Eq + ?Sized,
    {
        self.0.values_mut().find_map(|keys| {
            keys.values_mut()
                .find_map(|values| values.shift_remove(name))
        })
    }
}

/// <https://github.com/NSIS-Dev/nsis/blob/HEAD/Source/Platform.h#L672>
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
    MultiString = 7u32.to_le(),
}

impl RegType {
    #[expect(unused)]
    #[inline]
    pub const fn is_none(self) -> bool {
        matches!(self, Self::None)
    }

    #[inline]
    pub const fn is_string(self) -> bool {
        matches!(self, Self::String)
    }

    #[expect(unused)]
    #[inline]
    pub const fn is_expanded_string(self) -> bool {
        matches!(self, Self::ExpandedString)
    }

    #[inline]
    pub const fn is_binary(self) -> bool {
        matches!(self, Self::Binary)
    }

    #[inline]
    pub const fn is_dword(self) -> bool {
        matches!(self, Self::DWord)
    }

    #[expect(unused)]
    #[inline]
    pub const fn is_multi_string(self) -> bool {
        matches!(self, Self::MultiString)
    }
}
