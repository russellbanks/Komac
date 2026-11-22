use std::env;

use cynic_codegen::registration::SchemaRegistration;
use vergen_gix::{Emitter, Gix};

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

    emit_version();
}

/// Sets `KOMAC_VERSION` to the version shown by `--version`, e.g. `v2.17.2 (05f77093 2026-11-21)`,
/// leaving out the commit hash or date when they can't be determined
fn emit_version() {
    // The .cargo_vcs_info.json fallback lets builds from crates.io still know their commit, though
    // not its date
    let gix = Gix::builder()
        .sha(true)
        .commit_date(true)
        .vcs_info_fallback(true)
        .build();
    Emitter::default()
        .quiet()
        .add_instructions(&gix)
        .and_then(|emitter| emitter.emit_and_set())
        .expect("failed to determine the git commit");

    let version = env!("CARGO_PKG_VERSION");
    // vergen ignores the short option when falling back to .cargo_vcs_info.json, which only has the
    // full 40-character SHA, so shorten it to the 8 characters gix currently gives for this
    // repository
    let hash = vergen_var("VERGEN_GIT_SHA").map(|mut sha| {
        if sha.len() == 40 {
            sha.truncate(8);
        }
        sha
    });
    let date = vergen_var("VERGEN_GIT_COMMIT_DATE");

    let version = match (hash, date) {
        (Some(hash), Some(date)) => format!("v{version} ({hash} {date})"),
        (Some(part), None) | (None, Some(part)) => format!("v{version} ({part})"),
        (None, None) => format!("v{version}"),
    };
    println!("cargo:rustc-env=KOMAC_VERSION={version}");
}

/// Reads a variable set by vergen, ignoring the placeholder it substitutes when the
/// `VERGEN_IDEMPOTENT` environment variable is set
fn vergen_var(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .filter(|value| value != "VERGEN_IDEMPOTENT_OUTPUT")
}
