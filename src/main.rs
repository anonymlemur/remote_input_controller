pub mod input;
pub mod web_socket;

use std::{process, net::SocketAddr};
use log::{info, warn, error, debug};
use tray_icon::{TrayIconBuilder, Icon};
use tray_icon::menu::{Menu, MenuItem, MenuId, MenuEvent};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::event::{Event, StartCause};
use image::ImageReader;
use std::io::Cursor;
use tokio::sync::mpsc;
use crate::web_socket::Server;



// Commands sent from the main thread to the server thread
enum ServerCommand {
    Start,
    Stop,
    DisconnectClients,
}

// Status updates sent from the server thread to the main thread
pub enum ServerStatus {
    Started(SocketAddr),
    Stopped,
    ClientConnected(usize), // Number of connected clients
    ClientDisconnected(usize), // Number of connected clients
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    // Create communication channels
    let (server_command_tx, mut server_command_rx) = mpsc::channel::<ServerCommand>(10);
    let (server_status_tx, mut server_status_rx) = mpsc::channel::<ServerStatus>(10);
    let (client_disconnect_tx, _client_disconnect_rx) = mpsc::channel(10); // For Server::new

    // Menu setup (main thread only)
    let start_id = MenuId::new("start");
    let stop_id = MenuId::new("stop");
    let status_id = MenuId::new("status");
    let connect_id = MenuId::new("connect");
    let disconnect_id = MenuId::new("disconnect");
    let exit_id = MenuId::new("exit");

    let menu = Menu::new();
    let start_item = MenuItem::new("Start Server", true, None);
    let stop_item = MenuItem::new("Stop Server", false, None); // Initially disabled
    let status_item = MenuItem::new("Status", true, None);
    let connect_item = MenuItem::new("Connect", true, None);
    let disconnect_item = MenuItem::new("Disconnect", true, None);
    let exit_item = MenuItem::new("Exit", true, None);
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

    // Load and decode the icon using the 'image' crate
    let icon_data = include_bytes!("mouse.png"); // Assuming mouse.png is in the project root
    let reader = ImageReader::new(Cursor::new(icon_data)).with_guessed_format()?;
    let image = reader.decode()?;
    let image = image.into_rgba8(); // Convert to RGBA8 format
    let (width, height) = image.dimensions();
    let icon = Icon::from_rgba(image.into_raw(), width, height)?;

    // Build tray icon *after* event loop is created and with the icon
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Remote Input Controller")
        .with_icon(icon)
        .build()?;

    event_loop.run(move |event, event_loop_window_target| {
        event_loop_window_target.set_control_flow(ControlFlow::Wait);
        match event {
            Event::NewEvents(StartCause::Init) => {},
            Event::AboutToWait => {
                // Poll for tray/menu events within the event loop
                match MenuEvent::receiver().try_recv() {
                    Ok(event) => {
                    let id = event.id();
                    if id == &start_id {
                        info!("Menu: Start Server clicked");
                        // Send Start command to server thread
                        if let Err(e) = server_command_tx.try_send(ServerCommand::Start) {
                            error!("Failed to send Start command: {}", e);
                        } else {
                            info!("Sent Start command to server thread");
                        }
                    } else if id == &stop_id {
                        info!("Menu: Stop Server clicked");
                        // Send Stop command to server thread
                        if let Err(e) = server_command_tx.try_send(ServerCommand::Stop) {
                            error!("Failed to send Stop command: {}", e);
                        } else {
                            info!("Sent Stop command to server thread");
                        }
                    } else if id == &status_id {
                        info!("Menu: Status clicked");
                        // handle status - open status window
                    } else if id == &connect_id {
                        info!("Menu: Connect clicked");
                        // handle connect
                    } else if id == &disconnect_id {
                        info!("Menu: Disconnect clicked");
                        // Send DisconnectClients command to server thread
                        if let Err(e) = server_command_tx.try_send(ServerCommand::DisconnectClients) {
                            error!("Failed to send DisconnectClients command: {}", e);
                        } else {
                            info!("Sent DisconnectClients command to server thread");
                        }
                    } else if id == &exit_id {
                        info!("Menu: Exit clicked");
                        // Send Stop command before exiting
                        if let Err(e) = server_command_tx.try_send(ServerCommand::Stop) {
                            error!("Failed to send Stop command before exit: {}", e);
                        } else {
                            info!("Sent Stop command before exit");
                        }
                        event_loop_window_target.exit();
                    }

                    }
                    Err(_e) => {
                        // No menu event received this cycle
                    }
                }

                // Poll for server status updates from server thread
                while let Ok(status) = server_status_rx.try_recv() {
                    match status {
                        ServerStatus::Started(addr) => {
                            info!("Server started on: {}", addr);
                            // Update tray icon menu to show Stop and hide Start
                            start_item.set_enabled(false);
                            stop_item.set_enabled(true);
                            // TODO: Update status window with address and running state
                        }
                        ServerStatus::Stopped => {
                            info!("Server stopped");
                            // Update tray icon menu to show Start and hide Stop
                            start_item.set_enabled(true);
                            stop_item.set_enabled(false);
                            // TODO: Update status window with stopped state
                        }
                        ServerStatus::ClientConnected(count) => {
                            info!("Client connected. Total: {}", count);
                            // TODO: Update status window with connected clients count
                        }
                        ServerStatus::ClientDisconnected(count) => {
                            info!("Client disconnected. Total: {}", count);
                            // TODO: Update status window with connected clients count
                        }
                    }
                }
            },
            _ => {}
        }
    });
    // unreachable, but required for type
    Ok(())
}

// Remove unused enum
// #[derive(Debug)]
// enum ServerState {
//     Running,
//     Stopped,
// }


