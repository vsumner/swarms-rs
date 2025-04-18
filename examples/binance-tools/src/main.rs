use std::env;

use mcp_tools::BinanceMCPTools;
use rmcp::{
    ServiceExt,
    transport::{SseServer, stdio},
};
use tracing_subscriber::EnvFilter;

// examples/binance-mcp/src/main.rs
mod api;
mod auth;
mod mcp_tools;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting Binance MCP Server...");

    let (ctrlc_tx, ctrlc_rx) = tokio::sync::oneshot::channel::<()>();
    let (sse_cancel_tx, sse_cancel_rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        let sse_addr = env::var("BINANCE_MCP_SSE_ADDR")
            .unwrap_or("0.0.0.0:8000".parse().unwrap())
            .parse()
            .expect("Invalid SSE address");
        tracing::info!("Starting SSE server at {}", sse_addr);
        let ct = SseServer::serve(sse_addr)
            .await
            .expect("Failed to start SSE server")
            .with_service(BinanceMCPTools::new);

        tokio::select! {
            _ = ctrlc_rx => {
                ct.cancel();
                sse_cancel_tx.send(()).unwrap();
            }
        }
    });

    tracing::info!("Starting STDIO server...");
    let service = BinanceMCPTools::new().serve(stdio()).await?;

    tokio::signal::ctrl_c().await?;
    tracing::info!("Stopping Binance MCP Server...");

    ctrlc_tx.send(()).unwrap();
    service.cancel().await.unwrap();
    sse_cancel_rx.await.unwrap();

    tracing::info!("Binance MCP Server stopped.");
    Ok(())
}
