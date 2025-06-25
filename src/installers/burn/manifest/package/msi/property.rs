use serde::Deserialize;

#[expect(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MsiProperty<'manifest> {
    #[serde(rename = "@Id")]
    pub id: &'manifest str,
    #[serde(rename = "@Value")]
    pub value: &'manifest str,
    #[serde(rename = "@Condition")]
    pub condition: Option<&'manifest str>,
}

impl MsiProperty<'_> {
    /// Returns true if this property is [`ARPSYSTEMCOMPONENT`] with a value of 1.
    ///
    /// [`ARPSYSTEMCOMPONENT`]: https://learn.microsoft.com/windows/win32/msi/arpsystemcomponent
    pub fn is_arp_system_component(&self) -> bool {
        const ARP_SYSTEM_COMPONENT: &str = "ARPSYSTEMCOMPONENT";

        self.id == ARP_SYSTEM_COMPONENT && self.value.parse() == Ok(1)
    }
}
