use anyhow::Result;

use crate::{config::Config, server::start_server};

pub async fn handle_serve(config: &Config) -> Result<()> {
    start_server(config).await?;
    Ok(())
}

pub async fn handle_list(config: &Config) -> Result<()> {
    Ok(())
}
