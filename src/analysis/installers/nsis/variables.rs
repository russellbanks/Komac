use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
    fmt,
    hash::Hash,
};

use super::strings::PredefinedVar;

#[derive(Clone)]
pub struct Variables<'data>(HashMap<usize, Cow<'data, str>>);

impl<'data> Variables<'data> {
    /// There are 20 integer registers before predefined variables
    pub const NUM_REGISTERS: usize = 20;

    pub const NUM_INTERNAL_VARS: usize = Self::NUM_REGISTERS + PredefinedVar::num_vars();

    const INSTALL_DIR_INDEX: usize = Self::NUM_REGISTERS + PredefinedVar::InstDir as usize;

    #[inline]
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get<Q>(&self, index: &Q) -> Option<&str>
    where
        usize: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.0.get(index).map(Cow::as_ref)
    }

    pub fn insert<V>(&mut self, index: usize, variable: V) -> Option<Cow<'data, str>>
    where
        V: Into<Cow<'data, str>>,
    {
        self.0.insert(index, variable.into())
    }

    pub fn remove<Q>(&mut self, index: &Q) -> Option<Cow<'data, str>>
    where
        usize: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.0.remove(index)
    }

    pub fn install_dir(&self) -> Option<&str> {
        self.get(&Self::INSTALL_DIR_INDEX)
            .filter(|&dir| !dir.is_empty())
    }

    pub fn insert_install_dir<T>(&mut self, install_dir: T) -> Option<Cow<'_, str>>
    where
        T: Into<Cow<'data, str>>,
    {
        self.0.insert(Self::INSTALL_DIR_INDEX, install_dir.into())
    }
}

impl fmt::Debug for Variables<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
