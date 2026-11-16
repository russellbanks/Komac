use cynic_codegen::registration::SchemaRegistration;

const ICON: &str = "assets/branding/logo.ico";

fn main() {
    // cynic emits rerun-if-changed for the schema, which stops Cargo from rerunning this script
    // when any other file changes, so the icon has to be listed explicitly
    println!("cargo:rerun-if-changed={ICON}");

    cynic_codegen::register_schema("github")
        .from_sdl_file("assets/github.graphql")
        .and_then(SchemaRegistration::as_default)
        .expect("failed to register the GitHub GraphQL schema");
    windows_exe_info::icon::icon_ico(ICON);
    windows_exe_info::versioninfo::link_cargo_env();
}
