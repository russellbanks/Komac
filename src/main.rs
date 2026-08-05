#[cfg(all(feature = "openssl", feature = "rustls"))]
compile_error!("`openssl` and `rustls` are mutually exclusive. Please enable only one.");

mod analysis;
mod commands;
mod download;
mod editor;
mod github;
mod manifests;
mod prompts;
mod read;
mod terminal;
mod token;
mod traits;
mod update_state;

use clap::{Parser, crate_name};
use color_eyre::eyre::Result;
use commands::Commands;
use token::TokenManager;
use tracing::{Level, metadata::LevelFilter};
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .install()?;

    setup_logging();

    Cli::parse().command.run().await?;

    TokenManager::unset_default_store();

    Ok(())
}

fn setup_logging() {
    let indicatif_layer = IndicatifLayer::new();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(indicatif_layer.get_stderr_writer())
                .with_target(cfg!(debug_assertions))
                .without_time(),
        )
        .with(indicatif_layer)
        .with(
            filter::Targets::new()
                .with_default(LevelFilter::INFO)
                .with_target(crate_name!(), Level::TRACE),
        )
        .init();
}

#[derive(Parser)]
#[command(author, version, about, long_about = None, disable_version_flag = true)]
struct Cli {
    #[arg(short = 'v', short_alias = 'V', long, action = clap::builder::ArgAction::Version)]
    version: (),
    #[command(subcommand)]
    command: Commands,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::Cli;
    use crate::traits::AsciiExt;

    #[test]
    fn github_token_env_values_are_hidden_in_help() {
        fn assert_github_token_env_is_hidden(command: &clap::Command) {
            for arg in command.get_arguments() {
                if arg
                    .get_env()
                    .is_some_and(|env| env.contains_ignore_ascii_case("TOKEN"))
                {
                    assert!(
                        arg.is_hide_env_values_set(),
                        "command {:?} arg {:?} exposes token values in help",
                        command.get_name(),
                        arg.get_id()
                    );
                }
            }

            for subcommand in command.get_subcommands() {
                assert_github_token_env_is_hidden(subcommand);
            }
        }

        let command = Cli::command();
        assert_github_token_env_is_hidden(&command);
    }
}
