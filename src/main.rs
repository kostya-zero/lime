use std::process::exit;

use crate::{cli::Cli, commands::handle_serve, config::load_config};
use clap::Parser;

mod cli;
mod commands;
mod config;
mod server;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Loading configuration
    let config = load_config(&cli.config.unwrap()).unwrap_or_default();

    let result = match cli.command {
        cli::Commands::Serve => handle_serve(&config).await,
    };

    if let Err(e) = result {
        eprintln!("{e}");
        exit(1);
    }
}
