use std::sync::atomic::AtomicBool;

/// Indicates whether a client is currently connected to the WebSocket server.
/// Used to ensure only one client can be connected at a time.
pub static CONNECTED: AtomicBool = AtomicBool::new(false);

/// Global shutdown flag to signal all components to terminate.
/// When set to true, all async tasks should gracefully shut down.
pub static SHUTDOWN: AtomicBool = AtomicBool::new(false);