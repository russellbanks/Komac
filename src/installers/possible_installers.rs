use crate::installers::burn::Burn;
use crate::installers::inno::Inno;
use crate::installers::msi::Msi;
use crate::installers::msix_family::Msix;
use crate::installers::msix_family::bundle::MsixBundle;
use crate::installers::nsis::Nsis;
use crate::manifests::installer_manifest::Installer;

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
