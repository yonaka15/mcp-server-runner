use anyhow::{Result};
use log::{debug, error, info, warn};
use std::env;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};

use mcp_server_runner::{
    config::{self, model::ServerConfig},
    handle_connection, shutdown_signal, ProcessManager, CONNECTED, MESSAGE_BUFFER_SIZE, SHUTDOWN,
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .format_timestamp_millis()
        .format_target(true)
        .init();

    // コマンドライン引数を解析
    let args: Vec<String> = env::args().collect();
    let config_path = args.iter().skip(1).next().map(|s| s.as_str());

    // 設定の読み込み
    let config = config::load_config(config_path)?;

    // デフォルトサーバー設定の取得（所有権を取得してクローン）
    let default_server = config.default_server.clone()
        .and_then(|server_name| config.servers.get(&server_name).cloned())
        .ok_or_else(|| anyhow::anyhow!("No default server configuration found"))?;

    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await?;
    info!(
        "WebSocket server started on {} (Default server: {})",
        &addr,
        config.default_server.as_ref().unwrap_or(&"unknown".to_string())
    );

    // WebSocketサーバーとプロセス管理の設定 - Arcで共有可能にする
    let process_manager = Arc::new(Mutex::new(ProcessManager::new()));
    debug!("Process manager initialized");

    let shutdown_handle = tokio::spawn(shutdown_signal());
    debug!("Shutdown handler initialized");

    // サーバーループを起動（実際のサーバータスクを作成）
    let manager_clone = Arc::clone(&process_manager);
    let server_task = tokio::spawn(async move {
        run_server(
            listener, 
            manager_clone,
            default_server,
        ).await
    });

    // シャットダウンハンドラーとサーバーのいずれかが終了したら、全体をシャットダウン
    tokio::select! {
        result = server_task => {
            match result {
                Ok(Ok(())) => debug!("Server loop terminated normally"),
                Ok(Err(e)) => error!("Server loop terminated with error: {}", e),
                Err(e) => error!("Server task failed: {}", e),
            }
        },
        _ = shutdown_handle => {
            info!("Initiating shutdown sequence...");
            let mut pm = process_manager.lock().await;
            pm.shutdown().await;
            info!("Shutdown complete");
        }
    }

    Ok(())
}

/// WebSocketサーバーのメインループを実行
async fn run_server(
    listener: TcpListener,
    process_manager: Arc<Mutex<ProcessManager>>,
    server_config: ServerConfig,
) -> Result<()> {
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

        // プロセスを起動
        let process_tx = {
            let mut pm = process_manager.lock().await;
            match pm.start_process(&server_config.command, &server_config.args, &server_config.env, ws_tx.clone()).await {
                Ok(tx) => {
                    info!("Successfully started child process: {}", server_config.command);
                    tx
                }
                Err(e) => {
                    error!(
                        "Failed to start process: {}. Connection will be rejected",
                        e
                    );
                    continue;
                }
            }
        };

        debug!("Spawning connection handler for client: {}", addr);
        tokio::spawn(handle_connection(stream, process_tx, ws_rx));
        info!("Connection handler spawned for client: {}", addr);
    }

    Ok(())
}