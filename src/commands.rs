use std::net::SocketAddr;

// Commands sent from the main thread to the server thread
#[derive(Debug, Clone)]
pub enum ServerCommand {
    Start,
    Stop,
    DisconnectClients,
}

// Status updates sent from the server thread to the main thread
#[derive(Debug)]
pub enum ServerStatus {
    Started(SocketAddr),
    Stopped,
    ClientConnected(usize), // Number of connected clients
    ClientDisconnected(usize), // Number of connected clients
}
