mod message;

use anyhow::Result;
use log::info;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;
use std::sync::atomic::Ordering;
use futures_util::StreamExt;

use crate::state::CONNECTED;
use self::message::{handle_incoming_messages, handle_outgoing_messages};

/// Handle a new WebSocket connection
pub async fn handle_connection(
    stream: TcpStream,
    process_tx: mpsc::Sender<String>,
    ws_rx: mpsc::Receiver<String>,
) -> Result<()> {
    let addr = setup_connection(&stream)?;
    let ws_stream = accept_async(stream).await?;
    
    info!("WebSocket connection established: {}", addr);
    let (ws_writer, ws_reader) = ws_stream.split();

    let ws_to_process = handle_incoming_messages(ws_reader, process_tx);
    let process_to_ws = handle_outgoing_messages(ws_writer, ws_rx);

    tokio::select! {
        _ = ws_to_process => info!("WebSocket -> Process handling completed"),
        _ = process_to_ws => info!("Process -> WebSocket handling completed"),
    }
    
    cleanup_connection(addr);
    Ok(())
}

/// Set up initial connection state
fn setup_connection(stream: &TcpStream) -> Result<std::net::SocketAddr> {
    let addr = stream.peer_addr()?;
    CONNECTED.store(true, Ordering::SeqCst);
    Ok(addr)
}

/// Clean up connection state
fn cleanup_connection(addr: std::net::SocketAddr) {
    CONNECTED.store(false, Ordering::SeqCst);
    info!("Client disconnected: {}", addr);
}