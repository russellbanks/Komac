use winget_types::installer::Installer;

use crate::installers::{
    burn::Burn,
    inno::Inno,
    msi::Msi,
    msix_family::{Msix, bundle::MsixBundle},
    nsis::Nsis,
};

pub enum PossibleInstaller {
    Burn(Burn),
    Msi(Msi),
    Msix(Msix),
    MsixBundle(MsixBundle),
    Zip(Vec<Installer>),
    Inno(Inno),
    Nsis(Nsis),
    Other(Installer),
}

impl PossibleInstaller {
    pub fn installers(self) -> Vec<Installer> {
        match self {
            Self::Burn(burn) => vec![burn.installer],
            Self::Msi(msi) => vec![msi.installer],
            Self::Msix(msix) => vec![msix.installer],
            Self::MsixBundle(msix_bundle) => msix_bundle.installers,
            Self::Zip(installers) => installers,
            Self::Inno(inno) => inno.installers,
            Self::Nsis(nsis) => vec![nsis.installer],
            Self::Other(installer) => vec![installer],
        }
    }
}
