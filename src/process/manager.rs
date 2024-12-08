use anyhow::{Context, Result};
use log::{debug, error};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

use super::io::{handle_stdin, handle_stdout, handle_stderr};
use crate::constants::MESSAGE_BUFFER_SIZE;

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
        let child = self.spawn_process(program, args, env_vars)?;
        let (process_tx, process_rx) = mpsc::channel::<String>(MESSAGE_BUFFER_SIZE);
        
        self.setup_io_handlers(child, process_rx, websocket_tx)?;
        
        Ok(process_tx)
    }

    fn spawn_process(
        &mut self,
        program: &str,
        args: &[String],
        env_vars: &Vec<(String, String)>,
    ) -> Result<Child> {
        let mut command = Command::new(program);
        if !args.is_empty() {
            command.args(args);
        }
        
        for (key, value) in env_vars {
            command.env(key, value);
        }

        let child = command
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        Ok(child)
    }

    fn setup_io_handlers(
        &mut self,
        mut child: Child,
        process_rx: mpsc::Receiver<String>,
        websocket_tx: mpsc::Sender<String>,
    ) -> Result<()> {
        let stdin = child.stdin.take().context("Failed to get child stdin")?;
        let stdout = child.stdout.take().context("Failed to get child stdout")?;
        let stderr = child.stderr.take().context("Failed to get child stderr")?;

        self.child = Some(child);

        tokio::spawn(handle_stdin(stdin, process_rx));
        tokio::spawn(handle_stdout(stdout, websocket_tx));
        tokio::spawn(handle_stderr(stderr));

        Ok(())
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