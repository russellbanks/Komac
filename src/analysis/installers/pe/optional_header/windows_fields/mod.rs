mod windows_fields_32;
mod windows_fields_64;

pub use windows_fields_32::WindowsFields32;
pub use windows_fields_64::WindowsFields64;

pub type WindowsFields = WindowsFields64;
