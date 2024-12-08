use anyhow::{Context, Result};
use log::{debug, error};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

use crate::constants::MESSAGE_BUFFER_SIZE;
use crate::state::SHUTDOWN;

pub struct ProcessManager {
    child: Option<Child>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self { child: None }
    }

    pub async fn start_process(
        &mut self,
        program: &str,
        args: &[String],
        env_vars: &Vec<(String, String)>,
        websocket_tx: mpsc::Sender<String>,
    ) -> Result<mpsc::Sender<String>> {
        let mut command = Command::new(program);
        if !args.is_empty() {
            command.args(args);
        }
        for (key, value) in env_vars {
            command.env(key, value);
        }

        let mut child = command
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().context("Failed to get child stdin")?;
        let stdout = child.stdout.take().context("Failed to get child stdout")?;
        let stderr = child.stderr.take().context("Failed to get child stderr")?;

        self.child = Some(child);

        let (process_tx, mut process_rx) = mpsc::channel::<String>(MESSAGE_BUFFER_SIZE);

        let mut stdin = tokio::io::BufWriter::new(stdin);
        tokio::spawn(async move {
            while let Some(message) = process_rx.recv().await {
                if SHUTDOWN.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
                debug!("Sending to process: {}", message);
                if let Err(e) = stdin.write_all(message.as_bytes()).await {
                    error!("Error writing to child process stdin: {}", e);
                    break;
                }
                if let Err(e) = stdin.write_all(b"\n").await {
                    error!("Error writing newline: {}", e);
                    break;
                }
                if let Err(e) = stdin.flush().await {
                    error!("Error flushing stdin: {}", e);
                    break;
                }
            }
            debug!("Stdin handler finished");
        });

        // 標準出力の処理
        let websocket_tx_stdout = websocket_tx;
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();

            while let Ok(n) = reader.read_line(&mut line).await {
                if SHUTDOWN.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
                if n == 0 {
                    break;
                }
                let trimmed = line.trim().to_string();
                debug!("Received from process (stdout): {}", trimmed);
                if let Err(e) = websocket_tx_stdout.send(trimmed).await {
                    error!("Error sending to WebSocket: {}", e);
                    break;
                }
                line.clear();
            }
            debug!("Stdout handler finished");
        });

        // 標準エラー出力の処理（ログとして記録するだけ）
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();

            while let Ok(n) = reader.read_line(&mut line).await {
                if SHUTDOWN.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
                if n == 0 {
                    break;
                }
                let trimmed = line.trim().to_string();
                // 標準エラー出力はログとして記録するだけ
                error!("Process error: {}", trimmed);
                line.clear();
            }
            debug!("Stderr handler finished");
        });

        Ok(process_tx)
    }

    pub async fn shutdown(&mut self) {
        if let Some(mut child) = self.child.take() {
            debug!("Stopping child process...");
            if let Err(e) = child.kill().await {
                error!("Failed to stop child process: {}", e);
            }
            if let Err(e) = child.wait().await {
                error!("Error waiting for child process to exit: {}", e);
            }
            debug!("Child process stopped");
        }
    }
}