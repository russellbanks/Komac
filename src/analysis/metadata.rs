use winget_types::installer::Installer;

pub trait Metadata {
    fn installers(&self) -> Vec<Installer>;
}
