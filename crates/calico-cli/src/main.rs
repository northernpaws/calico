use clap::{Parser, Subcommand, crate_description, crate_version};
use color_eyre::eyre::Result;

use crate::commands::run::RunArgs;

mod cli;
mod commands;

pub(crate) mod gdb;

pub(crate) mod project;

#[derive(Parser)]
#[command(
	version,
	about = format!("{} v{}", crate_description!(), crate_version!()),
	styles(cli::style()),
	disable_colored_help(false),
	arg_required_else_help(true)
)]
struct CliArguments {
    #[command(subcommand)]
    pub subcommand: ToplevelCommmands,
}

#[derive(Subcommand)]
enum ToplevelCommmands {
    Run(RunArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("running calico");

    // Installs a custom error handler for panics
    // and error results from color_eyre.
    cli::install_error_handler()?;

    // Initialize an environment-configured
    // logger for the `log` framework.
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    // Parse the program arguments as CLI arguments and subcommands.
    let cli_args = CliArguments::parse();

    match &cli_args.subcommand {
        ToplevelCommmands::Run(args) => commands::run::execute(args).await?,
    }

    Ok(())
}
