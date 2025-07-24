mod qr_code;
pub mod input;
mod server;
mod app_state;
mod commands;
mod tray_menu;
mod ui_native;

use std::sync::Arc;
use log::{info, debug, error};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::event::{Event, StartCause};
use tokio::sync::mpsc;
use crate::server::Server;
use crate::app_state::AppState;
use crate::commands::{ServerCommand, ServerStatus};
use crate::tray_menu::{create_tray_menu, load_icon, handle_menu_event};

fn main() {
    // Print any panic from any thread
    std::panic::set_hook(Box::new(|panic_info| {
        println!("[PANIC] {}", panic_info);
    }));
    // Initialize logger with timestamp and level
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Millis))
        .format_module_path(true)
        .format_level(true)
        .init();

    info!("Starting Remote Input Controller main thread");

    // Create shared state
    let app_state = Arc::new(std::sync::Mutex::new(AppState::default()));
    
    // Create communication channels
    let (server_command_tx, server_command_rx) = mpsc::channel::<ServerCommand>(10);
    let (server_status_tx, mut server_status_rx) = mpsc::channel::<ServerStatus>(10);
    let (client_disconnect_tx, _client_disconnect_rx) = mpsc::channel(10); // For Server::new

    // Create tray menu
    let (menu, start_item, stop_item, start_id, stop_id, status_id, qr_id, connect_id, disconnect_id, exit_id) = create_tray_menu();

    // Start async server logic in a background thread with its own runtime
    let server_status_tx_clone = server_status_tx.clone();
    let client_disconnect_tx_clone = client_disconnect_tx.clone();
    std::thread::spawn(move || {
        let result = std::panic::catch_unwind(|| {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(async move {
                let mut server = Server::new(client_disconnect_tx_clone);
                if let Err(e) = server.run(server_command_rx, server_status_tx_clone).await {
                    error!("Server error: {}", e);
                }
            });
        });
        if let Err(err) = result {
            println!("[PANIC in server thread] {:?}", err);
        }
    });

    // Create tray icon
    let icon = load_icon("mouse.ico").expect("Failed to load icon");
    let tray_icon = tray_icon::TrayIconBuilder::new()
        .with_icon(icon)
        .with_tooltip("Remote Input Controller")
        .with_menu(Box::new(menu))
        .build()
        .expect("Failed to build tray icon");

    // Create event loop
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    info!("Starting winit event loop");
    info!("Event loop initialized");

    // Set up tray icon event handling
    let tray_icon_clone = tray_icon.clone();
    event_loop.run(move |event, event_loop| {
        match event {
            Event::NewEvents(StartCause::Init) => {
                info!("Application initialized");
            }
            Event::AboutToWait => {
                // Check for server status updates
                while let Ok(status) = server_status_rx.try_recv() {
                    debug!("Received ServerStatus: {:?}", status);
                    
                    let mut state = app_state.lock().unwrap();
                    match status {
                        ServerStatus::Started(addr) => {
                            info!("Server started on {}", addr);
                            println!("✅ Server started successfully on {}!", addr);
                            state.server_status = "Running".to_string();
                            state.server_address = Some(addr);
                            // Show Stop, hide Start
                            start_item.set_enabled(false);
                            stop_item.set_enabled(true);
                        }
                        ServerStatus::Stopped => {
                            info!("Server stopped");
                            println!("🛑 Server stopped.");
                            state.server_status = "Stopped".to_string();
                            state.server_address = None;
                            state.clients_connected = 0;
                            // Show Start, hide Stop
                            start_item.set_enabled(true);
                            stop_item.set_enabled(false);
                        }
                        ServerStatus::ClientConnected(count) => {
                            state.clients_connected = count;
                            debug!("Client connected, total: {}", count);
                        }
                        ServerStatus::ClientDisconnected(count) => {
                            state.clients_connected = count;
                            debug!("Client disconnected, total: {}", count);
                        }
                    }
                }

                // Check for tray menu events
                use tray_icon::menu::MenuEvent;
                if let Ok(event) = MenuEvent::receiver().try_recv() {
                    handle_menu_event(
                        &event.id,
                        &start_id,
                        &stop_id,
                        &status_id,
                        &qr_id,
                        &connect_id,
                        &disconnect_id,
                        &exit_id,
                        &server_command_tx,
                        &app_state,
                    );
                }
            }
            Event::LoopExiting => {
                info!("Event loop exiting");
                let _ = server_command_tx.send(ServerCommand::Stop);
            }
            _ => {}
        }
    }).expect("Event loop failed");
}
