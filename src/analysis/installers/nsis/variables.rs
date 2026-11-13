use std::{borrow::Borrow, collections::HashMap, fmt, hash::Hash};

use typed_path::Utf8WindowsPathBuf;

use super::strings::{NsisCow, NsisStr, PredefinedVar};

/// The NSIS variables (`$0`-`$9`, `$R0`-`$R9`, predefined variables, and user variables), keyed by
/// index and stored in the installer's native string encoding.
#[derive(Clone)]
pub struct Variables<'data>(HashMap<usize, NsisCow<'data>>);

impl<'data> Variables<'data> {
    /// There are 20 integer registers before predefined variables
    pub const NUM_REGISTERS: usize = 20;

    pub const NUM_INTERNAL_VARS: usize = Self::NUM_REGISTERS + PredefinedVar::num_vars();

    const INSTALL_DIR_INDEX: usize = Self::NUM_REGISTERS + PredefinedVar::InstDir as usize;

    /// Creates an empty set of variables.
    #[inline]
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Returns the value of the variable at `index`, or `None` if it has not been set.
    pub fn get<Q>(&self, index: &Q) -> Option<NsisStr<'_>>
    where
        usize: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.0.get(index).map(NsisCow::as_nsis_str)
    }

    /// Sets the variable at `index`, returning its previous value if it was set.
    pub fn insert<V>(&mut self, index: usize, variable: V) -> Option<NsisCow<'data>>
    where
        V: Into<NsisCow<'data>>,
    {
        self.0.insert(index, variable.into())
    }

    /// Unsets the variable at `index`, returning its previous value if it was set.
    pub fn remove<Q>(&mut self, index: &Q) -> Option<NsisCow<'data>>
    where
        usize: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.0.remove(index)
    }

    /// Returns the install directory decoded as a UTF-8 Windows path, if it is set and non-empty.
    pub fn install_dir(&self) -> Option<Utf8WindowsPathBuf> {
        self.get(&Self::INSTALL_DIR_INDEX)
            .filter(|dir| !dir.is_empty())
            .map(|dir| Utf8WindowsPathBuf::from(dir.to_string_lossy().into_owned()))
    }

    /// Sets `$INSTDIR`, returning its previous value if it was set.
    pub fn insert_install_dir<T>(&mut self, install_dir: T) -> Option<NsisCow<'data>>
    where
        T: Into<NsisCow<'data>>,
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
