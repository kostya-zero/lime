use std::process::exit;

use crate::{
    cli::Cli,
    config::{Config, load_config},
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
    let config = if let Ok(cfg) = load_config(&cli.config.unwrap()) {
        cfg
    } else {
        Config::default()
    };

    let result = match cli.command {
        cli::Commands::Serve => handle_serve(&config).await,
        cli::Commands::List => handle_list().await,
    };

    if let Err(e) = result {
        eprintln!("{e}");
        exit(1);
    }
}
