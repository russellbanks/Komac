use serde::Deserialize;

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct MsiProperty<'manifest> {
    #[serde(rename = "@Id")]
    pub id: &'manifest str,
    #[serde(rename = "@Value")]
    pub value: &'manifest str,
    #[serde(rename = "@Condition")]
    pub condition: Option<&'manifest str>,
}

impl<'manifest> MsiProperty<'manifest> {
    #[expect(dead_code)]
    #[must_use]
    #[inline]
    pub const fn id(&self) -> &'manifest str {
        self.id
    }

    #[expect(dead_code)]
    #[must_use]
    #[inline]
    pub const fn value(&self) -> &'manifest str {
        self.value
    }

    #[expect(dead_code)]
    #[must_use]
    #[inline]
    pub const fn condition(&self) -> Option<&'manifest str> {
        self.condition
    }

    /// Returns true if this property is [`ARPSYSTEMCOMPONENT`] with a value of 1.
    ///
    /// [`ARPSYSTEMCOMPONENT`]: https://learn.microsoft.com/windows/win32/msi/arpsystemcomponent
    #[must_use]
    pub fn is_arp_system_component(&self) -> bool {
        const ARP_SYSTEM_COMPONENT: &str = "ARPSYSTEMCOMPONENT";

        self.id == ARP_SYSTEM_COMPONENT && self.value.parse() == Ok(1)
    }
}
