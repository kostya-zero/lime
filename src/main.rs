use std::process::exit;

use crate::{
    cli::Cli,
    handlers::{handle_list, handle_serve},
};
use clap::Parser;

mod cli;
mod handlers;
mod server;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        cli::Commands::Serve => handle_serve().await,
        cli::Commands::List => handle_list().await,
    };

    if let Err(e) = result {
        println!("{e}");
        exit(1);
    }
}
