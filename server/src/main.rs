use service::start_server;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

const SERVER_ADDR: &str = "127.0.0.1:8080";

mod service;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    start_server(SERVER_ADDR).await?;

    Ok(())
}
