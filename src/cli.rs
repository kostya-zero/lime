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

    /// Path to the configuration file.
    #[arg(short, long, default_value = "lime.toml")]
    pub config: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start an HTML server.
    Serve,
}
