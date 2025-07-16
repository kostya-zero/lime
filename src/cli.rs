use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "lime",
    about = env!("CARGO_PKG_DESCRIPTION"),
    version = env!("CARGO_PKG_VERSION"),
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start an HTML server.
    Serve,

    /// See all pages that Lime has detected.
    List,
}
