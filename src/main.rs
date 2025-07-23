pub mod input;
pub mod web_socket;

use std::sync::{Arc, Mutex};
use std::process;
use std::thread;
use crate::web_socket::Server;
use std::net::SocketAddr;
use std::str::FromStr;
use uuid::Uuid;
use std::fs::File;
use sha2::{Sha256, Digest};
use tray_icon::{TrayIconBuilder, TrayIcon};
use tray_icon::menu::{Menu, MenuItem, MenuId, MenuEvent};
use rustls_pki_types::CertificateDer;

enum ServerCommand {
    DisconnectClients,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_address_str = "0.0.0.0:9000";
    let server_address = SocketAddr::from_str(server_address_str)?;
    let cert_path = "cert.pem";
    let key_path = "key.pem";

    // Load the certificate to get its fingerprint
    let cert_file = File::open(cert_path)?;
    let mut reader = std::io::BufReader::new(cert_file);
    let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut reader)
        .map(|res| res.map(|der| der.to_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    let cert_fingerprint = if let Some(cert) = certs.first() {
        let mut hasher = Sha256::new();
        hasher.update(cert);
        format!("{:x}", hasher.finalize())
    } else {
        "No certificate found".to_string()
    };

    // Move MenuId declarations to outer scope
    let start_id = MenuId::new("start");
    let stop_id = MenuId::new("stop");
    let status_id = MenuId::new("status");
    let connect_id = MenuId::new("connect");
    let disconnect_id = MenuId::new("disconnect");
    let exit_id = MenuId::new("exit");

    let mut menu = Menu::new();
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


    // Use correct channel type for Server::new
    let (client_disconnect_tx, mut rx) = tokio::sync::mpsc::channel::<Uuid>(32);
    let server = Arc::new(Mutex::new(Server::new(client_disconnect_tx.clone())));
    let server_state = Arc::new(Mutex::new(ServerState::Stopped));
    let cert_fingerprint_clone = cert_fingerprint.clone();

    // Build tray icon
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Remote Input Controller")
        .build()?;
    let tray_icon = Arc::new(tray_icon);

    // Set up menu event handler on the main thread (macOS requires GUI code on main thread)
    let server_clone: Arc<Mutex<Server>> = Arc::clone(&server);
    let server_state_clone = Arc::clone(&server_state);
    let cert_fingerprint_clone2 = cert_fingerprint.clone();
    let client_disconnect_tx_clone = client_disconnect_tx.clone();

    // Spawn async task for client disconnect handling
    tokio::spawn(async move {
        while let Some(_client_id) = rx.recv().await {
            // Handle client disconnect event
            let server_arc = Arc::clone(&server_clone);
            tokio::spawn(async move {
                // Lock, extract clients, drop lock, then await
                let clients = {
                    let mut server = server_arc.lock().unwrap();
                    let clients: Vec<_> = {
                        let mut map = server.connected_clients.lock().unwrap();
                        map.drain().map(|(_, ws)| ws).collect()
                    };
                    clients
                };
                for mut ws in clients {
                    if let Err(e) = ws.close(None).await {
                        eprintln!("Error disconnecting client: {:?}", e);
                    }
                }
            });
        }
    });

    // Main thread: menu event loop
    loop {
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
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // (No need to park the main thread; menu event loop above keeps it alive)
}

#[derive(Debug)]
enum ServerState {
    Running,
    Stopped,
}
