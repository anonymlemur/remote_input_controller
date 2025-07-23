pub mod input;
pub mod web_socket;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use log::{info, warn, error, debug};
use tray_icon::{TrayIconBuilder, Icon};
use tray_icon::menu::{Menu, MenuItem, MenuEvent, MenuId};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::event::{Event, StartCause};
use image::ImageReader;
use std::io::Cursor;
use tokio::sync::mpsc;
use crate::web_socket::Server;



// Commands sent from the main thread to the server thread
#[derive(Debug, Clone)]
pub enum ServerCommand {
    Start,
    Stop,
    DisconnectClients,
}

// Shared state between main thread and windows
#[derive(Clone)]
struct AppState {
    server_status: String,
    server_address: Option<SocketAddr>,
    clients_connected: usize,
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

// Status updates sent from the server thread to the main thread
#[derive(Debug)]
pub enum ServerStatus {
    Started(SocketAddr),
    Stopped,
    ClientConnected(usize), // Number of connected clients
    ClientDisconnected(usize), // Number of connected clients
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create shared state
    let app_state = Arc::new(Mutex::new(AppState::default()));
    
    // Initialize logger with timestamp and level
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Millis))
        .format_module_path(true)
        .format_level(true)
        .init();
    // Create communication channels
    let (server_command_tx, server_command_rx) = mpsc::channel::<ServerCommand>(10);
    let (server_status_tx, mut server_status_rx) = mpsc::channel::<ServerStatus>(10);
    let (client_disconnect_tx, _client_disconnect_rx) = mpsc::channel(10); // For Server::new

    // Create menu IDs
    let start_id = MenuId::new("start");
    let stop_id = MenuId::new("stop");
    let status_id = MenuId::new("status");
    let connect_id = MenuId::new("connect");
    let disconnect_id = MenuId::new("disconnect");
    let exit_id = MenuId::new("exit");

    let menu = Menu::new();
    
    // Create menu items with IDs
    let start_item = MenuItem::with_id(start_id.clone(), "Start Server", true, None);
    let stop_item = MenuItem::with_id(stop_id.clone(), "Stop Server", false, None);
    let status_item = MenuItem::with_id(status_id.clone(), "Status", true, None);
    let connect_item = MenuItem::with_id(connect_id.clone(), "Connect", true, None);
    let disconnect_item = MenuItem::with_id(disconnect_id.clone(), "Disconnect", true, None);
    let exit_item = MenuItem::with_id(exit_id.clone(), "Exit", true, None);

    // Create menu structure
    menu.append(&start_item).unwrap();
    menu.append(&stop_item).unwrap();
    menu.append(&status_item).unwrap();
    menu.append(&connect_item).unwrap();
    menu.append(&disconnect_item).unwrap();
    menu.append(&exit_item).unwrap();

    // Start async server logic in a background thread with its own runtime
    info!("Starting Remote Input Controller main thread");
    let server_status_tx_clone = server_status_tx.clone();
    let client_disconnect_tx_clone = client_disconnect_tx.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async move {
            let mut server = Server::new(client_disconnect_tx_clone);
            // Pass the receiver for commands and sender for status updates to the server
            server.run(server_command_rx, server_status_tx_clone).await.unwrap();
        });
    });

    // Use winit event loop for tray/menu events
    let event_loop = EventLoop::new().unwrap();

    // Update icon loading for Windows
    let icon_data = include_bytes!("mouse.png");
    let reader = ImageReader::new(Cursor::new(icon_data)).with_guessed_format()?;
    let image = reader.decode()?;
    let image = image.into_rgba8(); // Convert to RGBA8 format
    let (width, height) = image.dimensions();
    let icon = Icon::from_rgba(image.into_raw(), width, height)?;

    // Build tray icon *after* event loop is created and with the icon
    // Store the tray icon in a variable that lives as long as the event loop
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Remote Input Controller")
        .with_icon(icon)
        .build()?;

    // Keep the tray icon alive for the duration of the program
    let _ = event_loop.run(move |event, event_loop_window_target| -> () {
        // Move tray_icon into the closure to keep it alive
        let _tray = &tray_icon;
        event_loop_window_target.set_control_flow(ControlFlow::Wait);
        match event {
            Event::NewEvents(StartCause::Init) => {},
            Event::AboutToWait => {
                // Poll for tray/menu events within the event loop
                if let Ok(event) = MenuEvent::receiver().try_recv() {
                    debug!("Received menu event: {:?}", event);
                    let menu_id = event.id();
                    if menu_id == &start_id {
                        info!("Menu: Start Server clicked");
                        if let Err(e) = server_command_tx.try_send(ServerCommand::Start) {
                            error!("Failed to send Start command: {}", e);
                        }
                    } else if menu_id == &stop_id {
                        info!("Menu: Stop Server clicked");
                        if let Err(e) = server_command_tx.try_send(ServerCommand::Stop) {
                            error!("Failed to send Stop command: {}", e);
                        }
                    } else if menu_id == &status_id {
                        info!("Menu: Status clicked");
                        show_status_window(app_state.clone());
                    } else if menu_id == &connect_id {
                        info!("Menu: Connect clicked");
                        show_connect_window();
                    } else if menu_id == &disconnect_id {
                        info!("Menu: Disconnect clicked");
                        if let Err(e) = server_command_tx.try_send(ServerCommand::DisconnectClients) {
                            error!("Failed to send DisconnectClients command: {}", e);
                        }
                    } else if menu_id == &exit_id {
                        info!("Menu: Exit clicked");
                        if let Err(e) = server_command_tx.try_send(ServerCommand::Stop) {
                            error!("Failed to send Stop command before exit: {}", e);
                        }
                        event_loop_window_target.exit();
                    } else {
                        warn!("Unknown menu item clicked");
                    }
                }

                // Poll for server status updates from server thread
                let mut got_status = false;
                while let Ok(status) = server_status_rx.try_recv() {
                    got_status = true;
                    debug!("Received ServerStatus: {:?}", status);
                    // Update app state
                    if let Ok(mut state) = app_state.try_lock() {
                        match &status {
                            ServerStatus::Started(addr) => {
                                info!("Updating state to Running with address: {}", addr);
                                state.server_status = "Running".to_string();
                                state.server_address = Some(*addr);
                            }
                            ServerStatus::Stopped => {
                                info!("Updating state to Stopped");
                                state.server_status = "Stopped".to_string();
                                state.server_address = None;
                                state.clients_connected = 0;
                            }
                            ServerStatus::ClientConnected(count) => {
                                info!("Updating connected clients to: {}", count);
                                state.clients_connected = *count;
                            }
                            ServerStatus::ClientDisconnected(count) => {
                                info!("Updating connected clients to: {}", count);
                                state.clients_connected = *count;
                            }
                        }
                    } else {
                        error!("Failed to acquire state lock to update status");
                    }

                    // Update menu items
                    match status {
                        ServerStatus::Started(addr) => {
                            info!("Server started on: {}", addr);
                            start_item.set_enabled(false);
                            stop_item.set_enabled(true);
                        }
                        ServerStatus::Stopped => {
                            info!("Server stopped");
                            start_item.set_enabled(true);
                            stop_item.set_enabled(false);
                        }
                        ServerStatus::ClientConnected(count) => {
                            info!("Client connected. Total: {}", count);
                        }
                        ServerStatus::ClientDisconnected(count) => {
                            info!("Client disconnected. Total: {}", count);
                        }
                    }
                }
                if !got_status {
                    debug!("No ServerStatus received in this event loop iteration");
                }
            }
            _ => {}
        }
    });
    // unreachable, but required for type
    Ok(())
}

// Status window function
fn show_status_window(state: Arc<Mutex<AppState>>) {
    if let Ok(state) = state.try_lock() {
        // Use msg.exe to show status in a simple dialog
        let status_text = format!(
            "Server Status: {}\nServer Address: {}\nConnected Clients: {}",
            state.server_status,
            state.server_address.map_or("Not available".to_string(), |addr| addr.to_string()),
            state.clients_connected
        );
        
        info!("Showing status window with text:\n{}", status_text);
        
        let _ = std::process::Command::new("cmd")
            .args(&["/C", "msg", "*", &status_text])
            .output();
    } else {
        error!("Failed to acquire state lock to show status");
    }
}

fn show_connect_window() {
    // For now, just log that this was requested
    info!("Connect window requested - To be implemented");
}


