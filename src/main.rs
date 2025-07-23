pub mod input;
pub mod web_socket;

use std::sync::{Arc, Mutex};
use std::process;
use crate::web_socket::Server;
use std::net::SocketAddr;
use std::str::FromStr;
use uuid::Uuid;
use std::fs::File;
use sha2::{Sha256, Digest};
use tray_icon::TrayIconBuilder;
use tray_icon::menu::{Menu, MenuItem, MenuId, MenuEvent};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::event::{Event, StartCause};
use rustls_pki_types::CertificateDer;

enum ServerCommand {
    DisconnectClients,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Menu setup (main thread only)
    let start_id = MenuId::new("start");
    let stop_id = MenuId::new("stop");
    let status_id = MenuId::new("status");
    let connect_id = MenuId::new("connect");
    let disconnect_id = MenuId::new("disconnect");
    let exit_id = MenuId::new("exit");

    let menu = Menu::new();
    let start_item = MenuItem::new("Start Server", true, None);
    let stop_item = MenuItem::new("Stop Server", true, None);
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

    // Build tray icon (main thread only)
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Remote Input Controller")
        .build()?;

    // Start async server logic in a background thread with its own runtime
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async move {
            // TODO: Start your actual server logic here (e.g., run_server or run_tls_server)
            // For now, just park the thread
            std::thread::park();
        });
    });

    // Use winit event loop for tray/menu events
    let event_loop = EventLoop::new().unwrap();
    // According to winit 0.29 docs, closure should be (event, event_loop_window_target)
    // But E0308 error persists: expected EventLoopWindowTarget, found ControlFlow
    event_loop.run(move |event, event_loop_window_target| {
        event_loop_window_target.set_control_flow(ControlFlow::Wait);
        match event {
            Event::NewEvents(StartCause::Init) => {},
            Event::AboutToWait => {
                // Poll for tray/menu events
                if let Ok(event) = MenuEvent::receiver().try_recv() {
                    let id = event.id();
                    if id == &start_id {
                        // handle start
                    } else if id == &stop_id {
                        // handle stop
                    } else if id == &status_id {
                        // handle status
                    } else if id == &connect_id {
                        // handle connect
                    } else if id == &disconnect_id {
                        // handle disconnect
                    } else if id == &exit_id {
                        process::exit(0);
                    }
                }
            },
            _ => {}
        }
    });
    // unreachable, but required for type
    Ok(())
}

#[derive(Debug)]
enum ServerState {
    Running,
    Stopped,
}
