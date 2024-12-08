/// Size of the message buffer for communication channels.
/// This value affects the capacity of mpsc channels used for
/// process and WebSocket communication.
pub const MESSAGE_BUFFER_SIZE: usize = 100;