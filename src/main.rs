use anyhow::Result;
use config::Config;
use lazy_static::lazy_static;

mod client;
mod config;
mod log;
mod server;

lazy_static! {
    static ref CONFIG: Config = config::init();
}

#[tokio::main]
async fn main() -> Result<()> {
    // Logging
    let _guard = log::init()?;

    tracing::info!("准备完成");

    server::init().await
}
