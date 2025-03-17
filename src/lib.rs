pub mod config;
mod constants;
mod process;
mod shutdown;
mod state;
mod websocket;

// Re-export public API
pub use constants::MESSAGE_BUFFER_SIZE;
pub use process::ProcessManager;
pub use shutdown::shutdown_signal;
pub use websocket::handle_connection;
pub use state::{CONNECTED, SHUTDOWN};