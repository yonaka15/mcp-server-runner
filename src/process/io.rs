use log::{debug, error};
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

    while let Some(message) = process_rx.recv().await {
        if SHUTDOWN.load(Ordering::SeqCst) {
            break;
        }

        if let Err(e) = write_to_process(&mut writer, &message).await {
            error!("Error in stdin handling: {}", e);
            break;
        }
    }
    debug!("Stdin handler finished");
}

pub async fn handle_stdout(
    stdout: ChildStdout,
    websocket_tx: mpsc::Sender<String>,
) {
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();

    while let Ok(n) = reader.read_line(&mut line).await {
        if should_stop(n) {
            break;
        }

        let trimmed = line.trim().to_string();
        debug!("Received from process (stdout): {}", trimmed);

        if let Err(e) = websocket_tx.send(trimmed).await {
            error!("Error sending to WebSocket: {}", e);
            break;
        }
        line.clear();
    }
    debug!("Stdout handler finished");
}

pub async fn handle_stderr(stderr: ChildStderr) {
    let mut reader = BufReader::new(stderr);
    let mut line = String::new();

    while let Ok(n) = reader.read_line(&mut line).await {
        if should_stop(n) {
            break;
        }

        let trimmed = line.trim();
        // Log error messages from the process
        error!("Process error: {}", trimmed);
        line.clear();
    }
    debug!("Stderr handler finished");
}

async fn write_to_process(
    writer: &mut BufWriter<ChildStdin>,
    message: &str,
) -> tokio::io::Result<()> {
    debug!("Sending to process: {}", message);
    writer.write_all(message.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

fn should_stop(n: usize) -> bool {
    n == 0 || SHUTDOWN.load(Ordering::SeqCst)
}