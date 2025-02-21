use crate::shared::ManifestType;

pub trait Manifest {
    const SCHEMA: &'static str;

    const TYPE: ManifestType;
}
