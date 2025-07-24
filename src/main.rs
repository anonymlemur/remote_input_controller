mod qr_code;
pub mod input;
pub mod web_socket;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::{fs, path::Path};
use log::{info, warn, error, debug};
use tray_icon::{
    TrayIconBuilder,
    Icon
};
use tray_icon::menu::{Menu, MenuItem, MenuEvent, MenuId};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::event::{Event, StartCause};
use tokio::sync::mpsc;
use crate::web_socket::Server;
use eframe::NativeOptions;

// use qr_code; // Already imported as mod

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

fn main() {
    let qr_id = MenuId::new("qr");
    let qr_item = MenuItem::with_id(qr_id.clone(), "Show QR Code", true, None);
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
    menu.append(&qr_item).unwrap();
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
            if let Err(e) = server.run(server_command_rx, server_status_tx_clone).await {
                error!("Server error: {}", e);
            }
        });
    });

    // Create tray icon - try to load icon, but continue without it if missing
    let icon_path = "src/mouse.ico";
    let _tray_icon = match load_icon(icon_path) {
        Ok(icon) => {
            TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip("Remote Input Controller")
                .with_icon(icon)
                .build()
        }
        Err(e) => {
            warn!("Failed to load icon from {}: {}. Using default system icon.", icon_path, e);
            TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip("Remote Input Controller")
                .build()
        }
    }.unwrap();

    // Event loop for the tray icon
    let event_loop = EventLoop::new().unwrap();
    let menu_channel = MenuEvent::receiver();

    info!("Starting winit event loop");
    let _ = event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        // Handle winit events
        match event {
            Event::NewEvents(StartCause::Init) => {
                info!("Event loop initialized");
            }
            Event::AboutToWait => {
                // Check for menu events
                if let Ok(event) = menu_channel.try_recv() {
                    debug!("Menu event received: {:?}", event.id);
                    
                    if event.id == start_id {
                        info!("Start menu item clicked");
                        let _ = server_command_tx.send(ServerCommand::Start);
                    } else if event.id == stop_id {
                        info!("Stop menu item clicked");
                        let _ = server_command_tx.send(ServerCommand::Stop);
                    } else if event.id == status_id {
                        info!("Status menu item clicked");
                        show_status_window(app_state.clone());
                    } else if event.id == qr_id {
                        info!("QR Code menu item clicked");
                        let state = app_state.lock().unwrap();
                        if let Some(addr) = state.server_address {
                            let qr_data = format!("http://{}", addr);
                            match qr_code::display_qr_code(&qr_data) {
                                Ok(_) => {
                                    info!("QR code displayed successfully");
                                }
                                Err(e) => {
                                    error!("Failed to display QR code: {}", e);
                                    let error_msg = format!("Failed to display QR code: {}", e);
                                    let _ = show_error_window("QR Code Error", &error_msg);
                                }
                            }
                        } else {
                            warn!("Server is not running, cannot show QR code.");
                            let _ = show_error_window("Server Not Running", 
                                "Server is not running.\n\nPlease start the server to generate a QR code.");
                        }
                    } else if event.id == exit_id {
                        info!("Exit menu item clicked");
                        let _ = server_command_tx.send(ServerCommand::Stop);
                        elwt.exit();
                    }
                }

                // Check for server status updates
                let mut got_status = false;
                while let Ok(status) = server_status_rx.try_recv() {
                    got_status = true;
                    debug!("Received ServerStatus: {:?}", status);
                    
                    let mut state = app_state.lock().unwrap();
                    match status {
                        ServerStatus::Started(addr) => {
                            info!("Updating state to Running with address: {}", addr);
                            state.server_status = "Running".to_string();
                            state.server_address = Some(addr);
                        }
                        ServerStatus::Stopped => {
                            info!("Updating state to Stopped");
                            state.server_status = "Stopped".to_string();
                            state.server_address = None;
                            state.clients_connected = 0;
                        }
                        ServerStatus::ClientConnected(count) => {
                            info!("Client connected, total clients: {}", count);
                            state.clients_connected = count;
                        }
                        ServerStatus::ClientDisconnected(count) => {
                            info!("Client disconnected, total clients: {}", count);
                            state.clients_connected = count;
                        }
                    }
                }
                if !got_status {
                    // debug removed: No ServerStatus received in this event loop iteration
                }
            }
            Event::LoopExiting => {
                info!("Event loop exiting");
                let _ = server_command_tx.send(ServerCommand::Stop);
            }
            _ => {}
        }
    });
}

fn load_icon(path: &str) -> std::result::Result<Icon, Box<dyn std::error::Error>> {
    if Path::new(path).exists() {
        let image_bytes = fs::read(path)?;
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

/// Status window function that shows server status in a proper window
fn show_status_window(state: Arc<Mutex<AppState>>) {
    info!("Status window requested");
    
    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_title("Server Status")
            .with_resizable(false),
        ..Default::default()
    };
    
    let state_clone = state.clone();
    
    if let Err(e) = eframe::run_simple_native("Server Status", native_options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Server Status");
                ui.add_space(20.0);
                
                let state = state_clone.lock().unwrap();
                
                // Server status
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Status:").strong());
                    let status_text = match state.server_status.as_str() {
                        "Running" => egui::RichText::new("Running").color(egui::Color32::GREEN),
                        "Stopped" => egui::RichText::new("Stopped").color(egui::Color32::RED),
                        _ => egui::RichText::new(&state.server_status).color(egui::Color32::YELLOW),
                    };
                    ui.label(status_text);
                });
                
                ui.add_space(10.0);
                
                // Connected clients
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Connected Clients:").strong());
                    ui.label(egui::RichText::new(state.clients_connected.to_string()));
                });
                
                ui.add_space(10.0);
                
                // Server address
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Server Address:").strong());
                    if let Some(addr) = state.server_address {
                        ui.label(egui::RichText::new(addr.to_string()).monospace());
                    } else {
                        ui.label(egui::RichText::new("Not running").italics());
                    }
                });
                
                ui.add_space(30.0);
                
                // Close button
                if ui.button("Close").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });
    }) {
        error!("Failed to show status window: {}", e);
    }
}

/// Shows the connect window (placeholder for now)
fn show_connect_window() {
    info!("Connect window requested");
    
    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 200.0])
            .with_title("Connect to Server")
            .with_resizable(false),
        ..Default::default()
    };
    
    if let Err(e) = eframe::run_simple_native("Connect to Server", native_options, |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Connect to Server");
                ui.add_space(20.0);
                
                ui.label("Server Address:");
                ui.add(egui::TextEdit::singleline(&mut String::new()).hint_text("ws://localhost:8080"));
                
                ui.add_space(10.0);
                
                if ui.button("Connect").clicked() {
                    // TODO: Implement connect logic
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                
                ui.add_space(10.0);
                
                if ui.button("Cancel").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });
    }) {
        error!("Failed to show connect window: {}", e);
    }
}

/// Displays an error message in a proper window instead of a native dialog
fn show_error_window(title: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 200.0])
            .with_title(title)
            .with_resizable(false),
        ..Default::default()
    };

    let message = message.to_string();
    
    eframe::run_simple_native("Error", native_options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.heading("Error");
                ui.add_space(20.0);
                
                // Display the error message with word wrapping
                ui.label(egui::RichText::new(message.as_str()).text_style(egui::TextStyle::Body));
                
                ui.add_space(30.0);
                
                // Add an OK button to close the window
                if ui.button("OK").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });
    }).map_err(|e| e.into())
}
