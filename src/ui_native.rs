use std::sync::{Arc, Mutex};
use log::{info, error};
use crate::app_state::AppState;
use native_dialog::{DialogBuilder, MessageDialogBuilder, MessageLevel};

/// Displays server status using native system dialog
pub fn show_status_dialog(state: Arc<Mutex<AppState>>) {
    info!("Status dialog requested");
    
    let state = state.lock().unwrap();
    let status_text = format!(
        "Server Status: {}\n\nConnected Clients: {}\n\nServer Address: {}",
        state.server_status,
        state.clients_connected,
        state.server_address
            .as_ref()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "Not running".to_string())
    );


    let result = MessageDialogBuilder::default()
        .set_level(MessageLevel::Info)
        .set_title("Server Status")
        .set_text(&status_text)
        .alert()
        .show();

    if let Err(e) = result {
        error!("Failed to show status dialog: {}", e);
        // Fallback to console output
        println!("=== Server Status ===");
        println!("{}", status_text);
    }
}

/// Shows connect dialog (placeholder for now)
pub fn show_connect_dialog() {
    info!("Connect dialog requested");
    
    let result = MessageDialogBuilder::default()
        .set_level(MessageLevel::Info)
        .set_title("Connect to Server")
        .set_text("Connect functionality is not yet implemented.\n\nPlease use a WebSocket client to connect to the server.")
        .alert()
        .show();

    if let Err(e) = result {
        error!("Failed to show connect dialog: {}", e);
        println!("Connect functionality not yet implemented");
    }
}

/// Displays an error message using native system dialog
pub fn show_error_dialog(title: &str, message: &str) {
    info!("Error dialog requested: {}", title);
    
    let result = MessageDialogBuilder::default()
        .set_level(MessageLevel::Error)
        .set_title(title)
        .set_text(message)
        .alert()
        .show();

    if let Err(e) = result {
        error!("Failed to show error dialog: {}", e);
        // Fallback to console output
        eprintln!("ERROR: {} - {}", title, message);
    }
}
