use crate::installers::utils::registry::RegRoot;
use std::borrow::Cow;
use std::collections::HashMap;
use tracing::debug;

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

    pub fn set_value(
        &mut self,
        root: RegRoot,
        key: Cow<'data, str>,
        name: Cow<'data, str>,
        value: Cow<'data, str>,
    ) {
        debug!(?root, %key, %name, %value);
        self.0
            .entry(root)
            .or_default()
            .entry(key)
            .or_default()
            .insert(name, value);
    }

    pub fn remove_value(&mut self, name: &str) -> Option<Cow<'data, str>> {
        self.0
            .values_mut()
            .find_map(|keys| keys.values_mut().find_map(|values| values.remove(name)))
    }
}
