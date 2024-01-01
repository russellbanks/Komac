extern crate windows_exe_info;

use std::path::Path;

fn main() {
    windows_exe_info::icon::icon_ico(Path::new("assets/logo.ico"));
    windows_exe_info::versioninfo::link_cargo_env();
}
