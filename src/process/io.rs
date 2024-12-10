use log::{debug, error, info, warn};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{ChildStdin, ChildStdout, ChildStderr};
use tokio::sync::mpsc;
use std::sync::atomic::Ordering;

use crate::state::SHUTDOWN;

pub async fn handle_stdin(
    stdin: ChildStdin,
    mut process_rx: mpsc::Receiver<String>,
) {
    let mut writer = BufWriter::new(stdin);
    debug!("Started stdin handler for child process");

    while let Some(message) = process_rx.recv().await {
        if SHUTDOWN.load(Ordering::SeqCst) {
            debug!("Shutdown signal received, stopping stdin handler");
            break;
        }

        debug!("Received message to send to process. Length: {}", message.len());
        if let Err(e) = write_to_process(&mut writer, &message).await {
            error!("Error in stdin handling: {}. Message was: {}", e, message);
            break;
        }
        debug!("Successfully wrote message to process");
    }
    info!("Stdin handler finished");
}

pub async fn handle_stdout(
    stdout: ChildStdout,
    websocket_tx: mpsc::Sender<String>,
) {
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    debug!("Started stdout handler for child process");

    while let Ok(n) = reader.read_line(&mut line).await {
        if should_stop(n) {
            debug!("Stopping stdout handler: {}", 
                if n == 0 { "EOF reached" } else { "shutdown requested" });
            break;
        }

        let trimmed = line.trim().to_string();
        debug!("Received from process (stdout) - Length: {}, Content: {}", 
            trimmed.len(), trimmed);

        if let Err(e) = websocket_tx.send(trimmed).await {
            error!("Error sending to WebSocket: {}", e);
            break;
        }
        debug!("Successfully sent process output to WebSocket");
        line.clear();
    }
    info!("Stdout handler finished");
}

pub async fn handle_stderr(stderr: ChildStderr) {
    let mut reader = BufReader::new(stderr);
    let mut line = String::new();
    debug!("Started stderr handler for child process");

    while let Ok(n) = reader.read_line(&mut line).await {
        if should_stop(n) {
            debug!("Stopping stderr handler: {}", 
                if n == 0 { "EOF reached" } else { "shutdown requested" });
            break;
        }

        let trimmed = line.trim();
        warn!("Process stderr: {}", trimmed);
        line.clear();
    }
    info!("Stderr handler finished");
}

async fn write_to_process(
    writer: &mut BufWriter<ChildStdin>,
    message: &str,
) -> tokio::io::Result<()> {
    debug!("Writing to process - Length: {}, Content: {}", message.len(), message);
    writer.write_all(message.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    debug!("Successfully flushed message to process");
    Ok(())
}

fn should_stop(n: usize) -> bool {
    n == 0 || SHUTDOWN.load(Ordering::SeqCst)
}
