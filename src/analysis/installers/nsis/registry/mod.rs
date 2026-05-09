mod root;
mod r#type;

use std::{borrow::Borrow, collections::BTreeMap, fmt};

use itertools::{Itertools, Position};
pub use root::RegRoot;
pub use r#type::RegType;

type Key = String;

type ValueName = String;

pub type Value = String;

type Values = BTreeMap<ValueName, Value>;

type Keys = BTreeMap<Key, Values>;

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
pub struct Registry(BTreeMap<RegRoot, Keys>);

const CURRENT_VERSION_UNINSTALL: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall";

impl Registry {
    #[inline]
    pub const fn new() -> Self {
        Self(BTreeMap::new())
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
        K: Ord + ?Sized,
    {
        self.0.get_mut(&root)?.remove(key)
    }

    /// Removes a specific named value from a key within a given registry root.
    pub fn remove_value_name<K, N>(&mut self, root: RegRoot, key: &K, name: &N) -> Option<Value>
    where
        Key: Borrow<K>,
        ValueName: Borrow<N>,
        K: Ord + ?Sized,
        N: Ord + ?Sized,
    {
        self.0.get_mut(&root)?.get_mut(key)?.remove(name)
    }

    /// Returns a reference to the first occurrence of a value with the specified name across all
    /// registry roots and keys.
    pub fn get_value_by_name<N>(&self, name: &N) -> Option<&Value>
    where
        ValueName: Borrow<N>,
        N: Ord + ?Sized,
    {
        self.0
            .values()
            .find_map(|keys| keys.values().find_map(|values| values.get(name)))
    }

    /// Removes the first occurrence of a value with the specified name across all registry roots
    /// and keys.
    pub fn remove_value_by_name<N>(&mut self, name: &N) -> Option<Value>
    where
        ValueName: Borrow<N>,
        N: Ord + ?Sized,
    {
        self.0
            .values_mut()
            .find_map(|keys| keys.values_mut().find_map(|values| values.remove(name)))
    }
}

impl fmt::Display for Registry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (position, (root, keys)) in self.0.iter().with_position() {
            if !matches!(position, Position::First | Position::Only) {
                writeln!(f)?;
            }

            for (position, (key, values)) in keys.iter().with_position() {
                if !matches!(position, Position::First | Position::Only) {
                    writeln!(f)?;
                }

                writeln!(f, r"{root}\{key}")?;

                let max_name_len = values.keys().map(ValueName::len).max().unwrap_or_default();

                for (name, value) in values {
                    writeln!(f, "    {name:<width$} = {value}", width = max_name_len)?;
                }
            }
        }

        Ok(())
    }
}
