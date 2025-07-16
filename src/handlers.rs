use anyhow::Result;

use crate::server::start_server;

pub async fn handle_serve() -> Result<()> {
    let port: u16 = 3000;
    start_server(port).await?;
    Ok(())
}

pub async fn handle_list() -> Result<()> {
    Ok(())
}
