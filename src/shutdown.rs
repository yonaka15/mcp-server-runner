use log::info;
use tokio::signal;
use std::sync::atomic::Ordering;

use crate::state::SHUTDOWN;

/// Handles shutdown signals for the application.
/// Listens for Ctrl+C and termination signals (on Unix systems),
/// and sets the global shutdown flag when received.
pub async fn shutdown_signal() {
    wait_for_shutdown_signal().await;
    initiate_shutdown();
}

/// Waits for either Ctrl+C or termination signal.
async fn wait_for_shutdown_signal() {
    let ctrl_c = setup_ctrl_c();
    let terminate = setup_terminate();

    tokio::select! {
        _ = ctrl_c => info!("Ctrl+C received"),
        _ = terminate => info!("Termination signal received"),
    }
}

/// Sets up Ctrl+C signal handler.
async fn setup_ctrl_c() {
    signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C handler");
}

/// Sets up termination signal handler (Unix only).
#[cfg(unix)]
async fn setup_terminate() {
    signal::unix::signal(signal::unix::SignalKind::terminate())
        .expect("Failed to install signal handler")
        .recv()
        .await;
}

/// Placeholder for non-Unix systems.
#[cfg(not(unix))]
async fn setup_terminate() {
    std::future::pending::<()>().await
}

/// Initiates the shutdown process by setting the global shutdown flag.
fn initiate_shutdown() {
    info!("Initiating shutdown sequence");
    SHUTDOWN.store(true, Ordering::SeqCst);
}