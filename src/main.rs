use anyhow::{Context, Result};
use log::{error, info, warn};
use std::env;
use std::sync::atomic::Ordering;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use mcp_server_runner::{
    handle_connection, shutdown_signal, ProcessManager, MESSAGE_BUFFER_SIZE, CONNECTED, SHUTDOWN,
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let program = env::var("PROGRAM").context("PROGRAM environment variable not set")?;
    let args = env::var("ARGS")
        .unwrap_or_default()
        .split(',')
        .map(String::from)
        .collect::<Vec<_>>();

    let env_vars: Vec<(String, String)> = env::vars()
        .filter(|(key, _)| key != "PROGRAM" && key != "ARGS")
        .collect();

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let addr = format!("{}:{}", host, port);

    let listener = TcpListener::bind(&addr).await?;
    info!("WebSocket server started: {}", &addr);
    info!("Command to execute: {} {:?}", program, args);

    let mut process_manager = ProcessManager::new();

    let shutdown_handle = tokio::spawn(shutdown_signal());

    let server = async {
        while let Ok((stream, addr)) = listener.accept().await {
            if SHUTDOWN.load(Ordering::SeqCst) {
                break;
            }

            if CONNECTED.load(Ordering::SeqCst) {
                warn!("Connection already established. Rejecting new connection: {}", addr);
                continue;
            }

            info!("New client connection: {}", addr);

            let (ws_tx, ws_rx) = mpsc::channel(MESSAGE_BUFFER_SIZE);

            let process_tx = match process_manager
                .start_process(&program, &args, &env_vars, ws_tx.clone())
                .await
            {
                Ok(tx) => tx,
                Err(e) => {
                    error!("Failed to start process: {}", e);
                    continue;
                }
            };

            tokio::spawn(handle_connection(stream, process_tx, ws_rx));
        }
    };

    tokio::select! {
        _ = server => {},
        _ = shutdown_handle => {
            info!("Initiating shutdown...");
            process_manager.shutdown().await;
            info!("Shutdown complete");
        }
    }

    Ok(())
}