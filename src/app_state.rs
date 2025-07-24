use std::net::SocketAddr;

/// Shared state between main thread and windows
#[derive(Clone)]
pub struct AppState {
    pub server_status: String,
    pub server_address: Option<SocketAddr>,
    pub clients_connected: usize,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            server_status: "Stopped".to_string(),
            server_address: None,
            clients_connected: 0,
        }
    }
}
