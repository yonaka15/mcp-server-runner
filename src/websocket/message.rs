use std::sync::atomic::Ordering;
use futures_util::{SinkExt, StreamExt};
use log::{debug, error};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::sink::Sink;

use crate::state::SHUTDOWN;

pub async fn handle_incoming_messages<S>(
    mut reader: S,
    process_tx: mpsc::Sender<String>,
) where
    S: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    while let Some(msg) = reader.next().await {
        if SHUTDOWN.load(Ordering::SeqCst) {
            break;
        }

        match process_incoming_message(msg, &process_tx).await {
            Ok(should_break) => {
                if should_break {
                    break;
                }
            }
            Err(e) => {
                error!("Error processing incoming message: {}", e);
                break;
            }
        }
    }
}

pub async fn handle_outgoing_messages<S>(
    mut writer: S,
    mut ws_rx: mpsc::Receiver<String>,
) where
    S: Sink<Message> + Unpin,
    S::Error: std::fmt::Debug,
{
    while let Some(msg) = ws_rx.recv().await {
        if SHUTDOWN.load(Ordering::SeqCst) {
            break;
        }

        debug!("Sending process response: {}", msg);
        if let Err(e) = writer.send(Message::Text(msg)).await {
            error!("Error sending to WebSocket: {:?}", e);
            break;
        }
    }
}

async fn process_incoming_message(
    msg: Result<Message, tokio_tungstenite::tungstenite::Error>,
    process_tx: &mpsc::Sender<String>,
) -> Result<bool, Box<dyn std::error::Error>> {
    match msg {
        Ok(msg) => {
            if msg.is_close() {
                return Ok(true);
            }
            if let Ok(text) = msg.into_text() {
                debug!("Received from client: {}", text);
                process_tx.send(text).await?;
            }
        }
        Err(e) => {
            error!("Error receiving from WebSocket: {}", e);
            return Ok(true);
        }
    }
    Ok(false)
}