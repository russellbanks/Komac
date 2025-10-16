use winget_types::installer::Installer;

pub trait Installers {
    fn installers(&self) -> Vec<Installer>;
}
