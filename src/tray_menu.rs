use std::sync::{Arc, Mutex};
use log::{info, error, warn};
use tray_icon::Icon;
use tray_icon::menu::{Menu, MenuItem, MenuId};
use crate::app_state::AppState;
use crate::commands::ServerCommand;
use crate::ui_native::{show_status_dialog, show_connect_dialog, show_error_dialog};
use crate::qr_code;
use std::path::Path;

/// Loads an icon from a file path
pub fn load_icon(path: &str) -> Result<Icon, Box<dyn std::error::Error>> {
    if Path::new(path).exists() {
        let image_bytes = std::fs::read(path)?;
        let image = image::load_from_memory(&image_bytes)?;
        let rgba_image = image.to_rgba8();
        let (width, height) = rgba_image.dimensions();
        
        // Ensure dimensions are reasonable for a tray icon
        let icon_width = width.min(32);
        let icon_height = height.min(32);
        
        let icon = Icon::from_rgba(rgba_image.into_raw(), icon_width, icon_height)?;
        Ok(icon)
    } else {
        // Create a simple 16x16 red square as fallback
        let size = 16;
        let mut rgba = Vec::with_capacity((size * size * 4) as usize);
        
        for _ in 0..(size * size) {
            rgba.extend_from_slice(&[255, 0, 0, 255]); // Red pixel with alpha
        }
        
        let icon = Icon::from_rgba(rgba, size, size)?;
        Ok(icon)
    }
}

/// Creates the system tray menu structure
pub fn create_tray_menu() -> (Menu, MenuId, MenuId, MenuId, MenuId, MenuId, MenuId, MenuId) {
    let start_id = MenuId::new("start");
    let stop_id = MenuId::new("stop");
    let status_id = MenuId::new("status");
    let qr_id = MenuId::new("qr");
    let connect_id = MenuId::new("connect");
    let disconnect_id = MenuId::new("disconnect");
    let exit_id = MenuId::new("exit");

    let menu = Menu::new();
    
    // Create menu items with IDs
    let start_item = MenuItem::with_id(start_id.clone(), "Start Server", true, None);
    let stop_item = MenuItem::with_id(stop_id.clone(), "Stop Server", false, None);
    let status_item = MenuItem::with_id(status_id.clone(), "Status", true, None);
    let qr_item = MenuItem::with_id(qr_id.clone(), "Show QR Code", true, None);
    let connect_item = MenuItem::with_id(connect_id.clone(), "Connect", true, None);
    let disconnect_item = MenuItem::with_id(disconnect_id.clone(), "Disconnect", true, None);
    let exit_item = MenuItem::with_id(exit_id.clone(), "Exit", true, None);

    // Create menu structure
    menu.append(&start_item).unwrap();
    menu.append(&stop_item).unwrap();
    menu.append(&status_item).unwrap();
    menu.append(&qr_item).unwrap();
    menu.append(&connect_item).unwrap();
    menu.append(&disconnect_item).unwrap();
    menu.append(&exit_item).unwrap();

    (menu, start_id, stop_id, status_id, qr_id, connect_id, disconnect_id, exit_id)
}

/// Handles tray menu events
pub fn handle_menu_event(
    event_id: &MenuId,
    start_id: &MenuId,
    stop_id: &MenuId,
    status_id: &MenuId,
    qr_id: &MenuId,
    connect_id: &MenuId,
    disconnect_id: &MenuId,
    exit_id: &MenuId,
    server_command_tx: &tokio::sync::mpsc::Sender<ServerCommand>,
    app_state: &Arc<Mutex<AppState>>,
) {
    use log::debug;
    
    debug!("Menu event received: {:?}", event_id);
    
    if event_id == start_id {
        info!("Start menu item clicked");
        info!("Sending ServerCommand::Start to server thread");
        let _ = server_command_tx.send(ServerCommand::Start);
    } else if event_id == stop_id {
        info!("Stop menu item clicked");
        let _ = server_command_tx.send(ServerCommand::Stop);
    } else if event_id == status_id {
        info!("Status menu item clicked");
        show_status_dialog(app_state.clone());
    } else if event_id == qr_id {
        info!("QR Code menu item clicked");
        let state = app_state.lock().unwrap();
        debug!("Current server status: {}, address: {:?}", state.server_status, state.server_address);
        if let Some(addr) = state.server_address {
            let qr_data = format!("http://{}", addr);
            info!("Generating QR code for: {}", qr_data);
            match qr_code::display_qr_code(&qr_data) {
                Ok(_) => info!("QR code displayed successfully"),
                Err(e) => {
                    error!("Failed to display QR code: {}", e);
                    show_error_dialog("QR Code Error", &format!("Failed to display QR code: {}", e));
                }
            }
        } else {
            warn!("Server is not running, cannot show QR code.");
            show_error_dialog("QR Code Error", "Server is not running. Start the server first to show QR code.");
        }
    } else if event_id == connect_id {
        info!("Connect menu item clicked");
        show_connect_dialog();
    } else if event_id == disconnect_id {
        info!("Disconnect menu item clicked");
        let _ = server_command_tx.send(ServerCommand::DisconnectClients);
    } else if event_id == exit_id {
        info!("Exit menu item clicked");
        std::process::exit(0);
    }
}
