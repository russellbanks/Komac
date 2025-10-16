use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct MsiProperty {
    #[serde(rename = "@Id")]
    id: String,
    #[serde(rename = "@Value")]
    value: String,
    #[serde(rename = "@Condition")]
    condition: Option<String>,
}

impl MsiProperty {
    #[must_use]
    #[inline]
    pub const fn id(&self) -> &str {
        self.id.as_str()
    }

    #[expect(unused)]
    #[must_use]
    #[inline]
    pub const fn value(&self) -> &str {
        self.value.as_str()
    }

    #[expect(unused)]
    #[must_use]
    #[inline]
    pub fn condition(&self) -> Option<&str> {
        self.condition.as_deref()
    }

    /// Returns true if this property is [`ARPSYSTEMCOMPONENT`] with a value of 1.
    ///
    /// [`ARPSYSTEMCOMPONENT`]: https://learn.microsoft.com/windows/win32/msi/arpsystemcomponent
    #[must_use]
    pub fn is_arp_system_component(&self) -> bool {
        const ARP_SYSTEM_COMPONENT: &str = "ARPSYSTEMCOMPONENT";

        self.id() == ARP_SYSTEM_COMPONENT && self.value.parse() == Ok(1)
    }
}
