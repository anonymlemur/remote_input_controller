pub mod input;
pub mod web_socket;

use std::process;
use systray::{Application, MenuItem};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::runtime::Runtime;
use crate::web_socket::Server;
use std::net::SocketAddr;
use std::str::FromStr;
use qrcode_generator::QrCodeEcc;
use image::{Luma, ImageView};
use std::borrow::Cow;
use local_ip_address::local_ip;
use uuid::Uuid;
use std::fs::File;
use std::io::Write;
use native_dialog::MessageDialog;
use rustls_pemfile;
use rustls::Certificate;
use sha2::{Sha256, Digest};
use tokio::sync::mpsc;

enum ServerCommand {
    DisconnectClients,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_address_str = "0.0.0.0:9000";
    let server_address = SocketAddr::from_str(server_address_str)?;
    let cert_path = "cert.pem"; // Assuming certificates are in the root directory
    let key_path = "key.pem";

    // Load the certificate to get its fingerprint
    let cert_file = File::open(cert_path)?;
    let mut reader = std::io::BufReader::new(cert_file);
    let certs: Vec<Certificate> = rustls_pemfile::certs(&mut reader)?
        .into_iter()
        .map(Certificate)
        .collect();
    let cert_fingerprint = if let Some(cert) = certs.first() {
        let mut hasher = Sha256::new();
        hasher.update(&cert.0);
        format!("{:x}", hasher.finalize())
    } else {
        "No certificate found".to_string()
    };

    // Create a Tokio runtime for the server and a channel for commands
    let runtime = Arc::new(Mutex::new(Runtime::new()?));
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerCommand>();
    let server = Arc::new(Mutex::new(Server::new(tx.clone()))); // Pass the sender to the Server
    let server_state = Arc::new(Mutex::new(ServerState::Stopped));

    // Spawn a task to listen for commands and control the server
    let server_clone = Arc::clone(&server);
    runtime.lock().unwrap().spawn(async move {
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

    // Create the system tray application
    let mut app = Application::new()?;

    // Add menu items
    app.add_menu_item(MenuItem::new("Start Server", move |_| {
        let runtime = Arc::clone(&runtime);
        let server = Arc::clone(&server);
        let server_state = Arc::clone(&server_state);
        let cert_path = cert_path.to_string();
        let key_path = key_path.to_string();
        let server_address_str = server_address_str.to_string();
        runtime.lock().unwrap().spawn(async move {
            *server_state.lock().unwrap() = ServerState::Running;
            if let Err(e) = web_socket::run_server(&server_address_str, &cert_path, &key_path, server).await {
                eprintln!("Failed to start the server: {:?}", e);
                *server_state.lock().unwrap() = ServerState::Stopped;
            }
        });
    }))?;

    app.add_menu_item(MenuItem::new("Stop Server", move |_| {
        let server = Arc::clone(&server);
        let server_state = Arc::clone(&server_state);
        let mut server = server.lock().unwrap();
        if let Err(e) = server.stop() {
            eprintln!("Failed to stop the server: {:?}", e);
        } else {
            *server_state.lock().unwrap() = ServerState::Stopped;
        }
    }))?;

    app.add_menu_item(MenuItem::new("Status", move |_| {
        let server_state = Arc::clone(&server_state);
        let server = Arc::clone(&server);
        let state = server_state.lock().unwrap();
        let clients_count = server.lock().unwrap().get_connected_clients_count();
        let status_message = match *state {
            ServerState::Running => format!("Server Status: Running
Connected Clients: {}", clients_count),
            ServerState::Stopped => "Server Status: Stopped".to_string(),
        };
        MessageDialog::new()
            .set_title("Server Status")
            .set_text(&status_message)
            .show_info().unwrap();
    }))?;

    app.add_menu_item(MenuItem::new("Connect", move |_| {
        let server_address = server_address.to_string();
        let cert_fingerprint = cert_fingerprint.clone();
        match local_ip() {
            Ok(ip) => {
                let device_id = Uuid::new_v4().to_string();
                let connection_string = format!("wss://{}:{}/{}?cert_fingerprint={}", ip, server_address.split(':').last().unwrap_or("9000"), device_id, cert_fingerprint);
                match qrcode_generator::to_img_to_png(&connection_string, QrCodeEcc::High, 1024) {
                    Ok(qrcode_image_data) => {
                        let filename = format!("qrcode_{}.png", device_id);
                        match File::create(&filename).and_then(|mut file| file.write_all(&qrcode_image_data)) {
                            Ok(_) => {
                                MessageDialog::new()
                                    .set_title("QR Code Generated")
                                    .set_text(&format!("QR code saved to {}", filename))
                                    .show_info().unwrap();
                                // In a real application, you might want to open the file automatically
                            },
                            Err(e) => {
                                eprintln!("Failed to save QR code image: {:?}", e);
                                MessageDialog::new()
                                    .set_title("Error")
                                    .set_text(&format!("Failed to save QR code image: {:?}", e))
                                    .show_error().unwrap();
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to generate QR code: {:?}", e);
                        MessageDialog::new()
                            .set_title("Error")
                            .set_text(&format!("Failed to generate QR code: {:?}", e))
                            .show_error().unwrap();
                    }
                }
            },
            Err(e) => {
                eprintln!("Failed to get local IP address: {:?}", e);
                MessageDialog::new()
                    .set_title("Error")
                    .set_text(&format!("Failed to get local IP address: {:?}", e))
                    .show_error().unwrap();
            }
        }
    }))?;

    app.add_menu_item(MenuItem::new("Disconnect", move |_| {
        let tx = tx.clone();
        if let Err(e) = tx.send(ServerCommand::DisconnectClients) {
            eprintln!("Failed to send disconnect command: {:?}", e);
        }
    }))?;

    app.add_menu_item(MenuItem::new("Exit", move |app| {
        app.quit();
    }))?;

    // Run the tray application
    app.wait_for_message();

    Ok(())
}

enum ServerState {
    Running,
    Stopped,
}
