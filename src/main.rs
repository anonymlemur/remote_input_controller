pub mod input;
pub mod web_socket;

use std::process;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::runtime::Runtime;
use crate::web_socket::Server;
use std::net::SocketAddr;
use std::str::FromStr;
use qrcode_generator::QrCodeEcc;
use image::{Luma};
use std::borrow::Cow;
use local_ip_address::local_ip;
use uuid::Uuid;
use std::fs::File;
use std::io::Write;
use rustls_pemfile::certs;
use sha2::{Sha256, Digest};
use tokio::sync::mpsc;
use tray_icon::{TrayIconBuilder, TrayIcon};
use tray_icon::menu::{Menu, MenuItem, MenuId};
use std::sync::mpsc as std_mpsc;
use tokio_rustls::rustls::{Certificate, PrivateKey};

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
    let certs: Vec<Vec<u8>> = rustls_pemfile::certs(&mut reader)
        .collect::<Result<_, _>>()?;
    let cert_fingerprint = if let Some(cert) = certs.first() {
        let mut hasher = Sha256::new();
        hasher.update(cert);
        format!("{:x}", hasher.finalize())
    } else {
        "No certificate found".to_string()
    };

    // Correct tray menu creation for tray-icon 0.21.0 (uses muda under the hood)
    // Remove MenuId from MenuItem::new, use None for accelerator, and set the id using set_id()
    let mut menu = Menu::new();
    let mut start_item = MenuItem::new("Start Server", true, None);
    start_item.set_id(start_id.clone());
    menu.append(&start_item);
    let mut stop_item = MenuItem::new("Stop Server", true, None);
    stop_item.set_id(stop_id.clone());
    menu.append(&stop_item);
    let mut status_item = MenuItem::new("Status", true, None);
    status_item.set_id(status_id.clone());
    menu.append(&status_item);
    let mut connect_item = MenuItem::new("Connect", true, None);
    connect_item.set_id(connect_id.clone());
    menu.append(&connect_item);
    let mut disconnect_item = MenuItem::new("Disconnect", true, None);
    disconnect_item.set_id(disconnect_id.clone());
    menu.append(&disconnect_item);
    let mut exit_item = MenuItem::new("Exit", true, None);
    exit_item.set_id(exit_id.clone());
    menu.append(&exit_item);

    // Build tray icon and set up event handler
    let server = Arc::new(Mutex::new(Server::new(tokio::sync::mpsc::channel(32).0)));
    let server_state = Arc::new(Mutex::new(ServerState::Stopped));
    let cert_fingerprint_clone = cert_fingerprint.clone();
    let tx = tokio::sync::mpsc::unbounded_channel::<ServerCommand>().0;
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Remote Input Controller")
        .build()?;
    let tray_icon = Arc::new(tray_icon);

    // Set up menu event handler
    let server_clone = Arc::clone(&server);
    let server_state_clone = Arc::clone(&server_state);
    let cert_fingerprint_clone2 = cert_fingerprint_clone.clone();
    let tx_clone = tx.clone();
    tray_icon.set_menu_event_handler(move |event| {
        let id = event.id();
        if id == &start_id {
            let server = Arc::clone(&server_clone);
            let server_state = Arc::clone(&server_state_clone);
            let cert_path = cert_path.to_string();
            let key_path = key_path.to_string();
            let server_address_str = server_address_str.to_string();
            tokio::spawn(async move {
                *server_state.lock().unwrap() = ServerState::Running;
                if let Err(e) = web_socket::run_server(&server_address_str, &cert_path, &key_path, server).await {
                    eprintln!("Failed to start the server: {:?}", e);
                    *server_state.lock().unwrap() = ServerState::Stopped;
                }
            });
        } else if id == &stop_id {
            let mut server = server_clone.lock().unwrap();
            if let Err(e) = server.stop() {
                eprintln!("Failed to stop the server: {:?}", e);
            } else {
                *server_state_clone.lock().unwrap() = ServerState::Stopped;
            }
        } else if id == &status_id {
            let state = server_state_clone.lock().unwrap();
            let clients_count = server_clone.lock().unwrap().get_connected_clients_count();
            println!("Server Status: {:?}, Connected Clients: {}", *state, clients_count);
        } else if id == &connect_id {
            match local_ip() {
                Ok(ip) => {
                    let device_id = Uuid::new_v4().to_string();
                    let connection_string = format!("wss://{}:{}/{}?cert_fingerprint={}", ip, server_address_str.split(':').last().unwrap_or("9000"), device_id, cert_fingerprint_clone2);
                    match qrcode_generator::to_png_to_vec(&connection_string, QrCodeEcc::High, 1024) {
                        Ok(qrcode_image_data) => {
                            let filename = format!("qrcode_{}.png", device_id);
                            match File::create(&filename).and_then(|mut file| file.write_all(&qrcode_image_data)) {
                                Ok(_) => println!("QR code saved to {}", filename),
                                Err(e) => eprintln!("Failed to save QR code image: {:?}", e),
                            }
                        },
                        Err(e) => eprintln!("Failed to generate QR code: {:?}", e),
                    }
                },
                Err(e) => eprintln!("Failed to get local IP address: {:?}", e),
            }
        } else if id == &disconnect_id {
            if let Err(e) = tx_clone.send(ServerCommand::DisconnectClients) {
                eprintln!("Failed to send disconnect command: {:?}", e);
            }
        } else if id == &exit_id {
            process::exit(0);
        }
    });

    // Server command handler
    let server_clone = Arc::clone(&server);
    let mut rx = tx.subscribe();
    tokio::spawn(async move {
        while let Some(command) = rx.recv().await {
            match command {
                ServerCommand::DisconnectClients => {
                    if let Err(e) = server_clone.lock().unwrap().disconnect_clients().await {
                        eprintln!("Error disconnecting clients: {:?}", e);
                    }
                }
            }
        }
    });

    // Keep the main thread alive
    loop {
        std::thread::park();
    }
}

#[derive(Debug)]
enum ServerState {
    Running,
    Stopped,
}
