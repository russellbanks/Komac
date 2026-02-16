mod advanced;
pub mod burn;
mod exe;
pub mod inno;
mod msi;
pub mod msix_family;
pub mod nsis;
pub mod utils;
mod zip;

pub use advanced::AdvancedInstaller;
pub use burn::Burn;
pub use exe::Exe;
pub use msi::Msi;
pub use nsis::Nsis;
pub use zip::Zip;
