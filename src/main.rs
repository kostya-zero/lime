use std::process::exit;

use crate::{
    cli::Cli,
    config::load_config,
    handlers::{handle_list, handle_serve},
};
use clap::Parser;

mod cli;
mod config;
mod handlers;
mod server;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Loading configuration
    let config = load_config(&cli.config.unwrap()).unwrap_or_default();

    let result = match cli.command {
        cli::Commands::Serve => handle_serve(&config).await,
        cli::Commands::List => handle_list(&config).await,
    };

    if let Err(e) = result {
        eprintln!("{e}");
        exit(1);
    }
}
