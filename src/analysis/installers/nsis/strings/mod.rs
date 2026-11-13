mod ansi;
pub mod code;
mod nsis;
mod predefined;
pub mod shell;
pub mod var;
mod wide;

pub use ansi::AnsiStr;
use ansi::{AnsiChar, AnsiString, decode_char, encode_char};
pub use nsis::{NsisChar, NsisCow, NsisStr, NsisString};
pub use predefined::PredefinedVar;
use wide::{WideChar, WideStr, WideString};
