use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use std::env;
use std::sync::atomic::Ordering;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use mcp_server_runner::{
    handle_connection, shutdown_signal, ProcessManager, CONNECTED, MESSAGE_BUFFER_SIZE, SHUTDOWN,
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .format_timestamp_millis()
        .format_target(true)
        .init();

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
    info!(
        "WebSocket server started: {} (Program: {}, Args: {:?})",
        &addr, program, args
    );

    let mut process_manager = ProcessManager::new();
    debug!("Process manager initialized");

    let shutdown_handle = tokio::spawn(shutdown_signal());
    debug!("Shutdown handler initialized");

    let server = async {
        while let Ok((stream, addr)) = listener.accept().await {
            if SHUTDOWN.load(Ordering::SeqCst) {
                info!("Shutdown signal received, stopping server");
                break;
            }

            if CONNECTED.load(Ordering::SeqCst) {
                warn!(
                    "Connection rejected: Already have an active connection from: {}",
                    addr
                );
                continue;
            }

            info!("New client connection accepted: {}", addr);
            debug!(
                "Client connection details - Local addr: {:?}, Peer addr: {:?}",
                stream.local_addr(),
                stream.peer_addr()
            );

            let (ws_tx, ws_rx) = mpsc::channel(MESSAGE_BUFFER_SIZE);
            debug!(
                "Created message channels with buffer size: {}",
                MESSAGE_BUFFER_SIZE
            );

            let process_tx = match process_manager
                .start_process(&program, &args, &env_vars, ws_tx.clone())
                .await
            {
                Ok(tx) => {
                    info!("Successfully started child process");
                    tx
                }
                Err(e) => {
                    error!(
                        "Failed to start process: {}. Connection will be rejected",
                        e
                    );
                    continue;
                }
            };

            debug!("Spawning connection handler for client: {}", addr);
            tokio::spawn(handle_connection(stream, process_tx, ws_rx));
            info!("Connection handler spawned for client: {}", addr);
        }
    };

    tokio::select! {
        _ = server => {
            debug!("Server loop terminated");
        },
        _ = shutdown_handle => {
            info!("Initiating shutdown sequence...");
            process_manager.shutdown().await;
            info!("Shutdown complete");
        }
    }

    Ok(())
}
