extern crate windows_exe_info;

use cynic_codegen::registration::SchemaRegistration;

fn main() {
    cynic_codegen::register_schema("github")
        .from_sdl_file("assets/github.graphql")
        .and_then(SchemaRegistration::as_default)
        .unwrap();
    windows_exe_info::icon::icon_ico("assets/logo.ico");
    windows_exe_info::versioninfo::link_cargo_env();
}
