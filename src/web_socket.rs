use log::{info, warn, error};
use enigo::{Enigo, Settings};
use futures_util::stream::StreamExt;
use serde_json::Error as JsonError;
use std::{
    collections::HashMap,
    fs::File,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
 
};
use tokio::{net::{TcpListener, TcpStream}, sync::{oneshot, mpsc}};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use tokio_rustls::{
    rustls::{
        ServerConfig,
    },
    TlsAcceptor,
};
// use tokio_rustls::server::TlsStream; // Not currently used
use uuid::Uuid;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};

pub mod input_types;
use input_types::*;
use crate::ServerCommand;
use crate::ServerStatus;

pub struct Server {
    listener: Option<TcpListener>,
    shutdown_sender: Option<oneshot::Sender<()>>,
    pub connected_clients: Arc<Mutex<HashMap<Uuid, ()>>>,
    client_disconnect_sender: mpsc::Sender<Uuid>,
}

impl Server {
    pub fn new(client_disconnect_sender: mpsc::Sender<Uuid>) -> Self {
        Server {
            listener: None,
            shutdown_sender: None,
            connected_clients: Arc::new(Mutex::new(HashMap::new())),
            client_disconnect_sender,
        }
    }

    pub async fn run(
        &mut self,
        mut command_rx: mpsc::Receiver<ServerCommand>,
        status_tx: mpsc::Sender<ServerStatus>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let addr = "127.0.0.1:8080"; // TODO: Make address configurable
        
        // Try to load certificates, but fall back to HTTP if they don't exist
        let cert_path = "cert.pem";
        let key_path = "key.pem";
        
        let use_tls = Path::new(cert_path).exists() && Path::new(key_path).exists();
        let config = if use_tls {
            info!("TLS certificates found, using HTTPS");
            let certs = load_certs(cert_path)?;
            let key = load_private_key(key_path)?;
            Some(Arc::new(ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, key)?))
        } else {
            warn!("TLS certificates not found, using HTTP (insecure)");
            None
        };

        let mut running = false;
        loop {
            tokio::select! {
                Some(command) = command_rx.recv() => {
                    match command {
                        ServerCommand::Start => {
                            if self.listener.is_none() {
                                info!("Starting server...");
                                let status_tx_clone = status_tx.clone();
                                match self.start_http(addr, config.clone(), status_tx_clone).await {
                                    Ok(_) => {
                                        running = true;
                                        info!("Server started");
                                    }
                                    Err(e) => error!("Error starting server: {}", e),
                                }
                            } else {
                                warn!("Server is already running.");
                            }
                        }
                        ServerCommand::Stop => {
                            if self.listener.is_some() {
                                info!("Stopping server...");
                                self.stop()?;
                                status_tx.send(ServerStatus::Stopped).await?;
                                running = false;
                                info!("Server stopped");
                            } else {
                                warn!("Server is not running.");
                            }
                        }
                        ServerCommand::DisconnectClients => {
                            info!("Disconnecting clients...");
                            self.disconnect_clients().await?;
                            let count = self.get_connected_clients_count();
                            status_tx.send(ServerStatus::ClientDisconnected(count)).await.ok();
                            info!("Clients disconnected.");
                        }
                    }
                }
                // Add a small delay to prevent busy-looping if no commands are received
                _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                    if running && self.listener.is_none() {
                        status_tx.send(ServerStatus::Stopped).await.ok();
                        running = false;
                    }
                }
            }
        }
    }

    async fn start(
        &mut self,
        addr: &str,
        config: Arc<ServerConfig>,
        status_tx: mpsc::Sender<ServerStatus>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.start_http(addr, Some(config), status_tx).await
    }

    async fn start_http(
        &mut self,
        addr: &str,
        config: Option<Arc<ServerConfig>>,
        status_tx: mpsc::Sender<ServerStatus>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        self.listener = Some(listener);
        let (tx, rx) = oneshot::channel();
        self.shutdown_sender = Some(tx);

        let enigo = Arc::new(Mutex::new(Enigo::new(&Settings::default()).unwrap()));
        let stop_move_flag = Arc::new(AtomicBool::new(false));
        let connected_clients = Arc::clone(&self.connected_clients);
        let client_disconnect_sender = self.client_disconnect_sender.clone();
        let status_tx_main = status_tx.clone();
        let listener = self.listener.as_ref().unwrap();

        // Send started status after binding
        status_tx.send(ServerStatus::Started(addr.parse()?)).await.ok();

        tokio::select! {
            res = async {
                while let Ok((stream, _)) = listener.accept().await {
                    let enigo_clone = enigo.clone();
                    let stop_move_flag_clone = stop_move_flag.clone();
                    let connected_clients_clone = connected_clients.clone();
                    let client_disconnect_sender_clone = client_disconnect_sender.clone();
                    let status_tx_clone = status_tx_main.clone();
                    let client_id = Uuid::new_v4();

                    // Insert client and send status update BEFORE spawning
                    {
                        let mut clients = connected_clients_clone.lock().unwrap();
                        clients.insert(client_id, ());
                        let count = clients.len();
                        status_tx_clone.send(ServerStatus::ClientConnected(count)).await.ok();
                    }

                    let config_clone = config.clone();
                    tokio::spawn(async move {
                        if let Err(e) = if let Some(tls_config) = config_clone {
                            // TLS connection
                            let acceptor = TlsAcceptor::from(tls_config);
                            handle_connection(
                                stream,
                                enigo_clone,
                                stop_move_flag_clone,
                                acceptor,
                                client_id,
                                client_disconnect_sender_clone
                            ).await
                        } else {
                            // HTTP connection (plain TCP)
                            handle_connection_http(
                                stream,
                                enigo_clone,
                                stop_move_flag_clone,
                                client_id,
                                client_disconnect_sender_clone
                            ).await
                        } {
                            error!("Error handling connection: {}", e);
                        }
                        // Remove the client from the map and send status update AFTER connection ends
                        let count = {
                            let mut clients = connected_clients_clone.lock().unwrap();
                            clients.remove(&client_id);
                            clients.len()
                        };
                        status_tx_clone.send(ServerStatus::ClientDisconnected(count)).await.ok();
                    });
                }
                Ok::<(), Box<dyn std::error::Error>>(())
            } => {
                if let Err(e) = res {
                    error!("Server error: {}", e);
                }
            }
            _ = rx => {
                info!("Server shutdown signal received");
            }
        }
        self.disconnect_clients().await.unwrap_or_else(|e| error!("Error disconnecting clients: {:?}", e));
        status_tx.send(ServerStatus::Stopped).await.ok();
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(tx) = self.shutdown_sender.take() {
            tx.send(()).map_err(|_| "Failed to send shutdown signal")?;
            self.listener.take(); // Drop the listener to stop accepting new connections
        }
        Ok(())
    }

    pub async fn disconnect_clients(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Remove all clients from the map (no websockets to close)
        let mut map = self.connected_clients.lock().unwrap();
        map.clear();
        Ok(())
    }

    pub fn get_connected_clients_count(&self) -> usize {
        self.connected_clients.lock().unwrap().len()
    }

    // Removed run_tls_server as run function now handles loading certs and keys
    // pub async fn run_tls_server(
    //     &mut self,
    //     addr: &str,
    //     cert_path: &str,
    //     key_path: &str,
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     // Load certificate and key using correct types
    //     let certs = load_certs(cert_path)?;
    //     let key = load_private_key(key_path)?;
    //     let config = ServerConfig::builder()
    //         .with_no_client_auth()
    //         .with_single_cert(certs, key)?;
    //     let config = Arc::new(config);

    //     // (Unused) let connected_clients = self.connected_clients.clone();
    //     // (Unused) let client_disconnect_sender = self.client_disconnect_sender.clone();
    //     self.start(addr, config).await
    // }
}

async fn handle_connection(
    stream: TcpStream,
    enigo: Arc<Mutex<Enigo>>,
    stop_move_flag: Arc<AtomicBool>,
    acceptor: TlsAcceptor,
    client_id: Uuid,
    client_disconnect_sender: mpsc::Sender<Uuid>,
) -> Result<(), Box<dyn std::error::Error>> {
    let stream = acceptor.accept(stream).await?;
    let mut ws = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(err) => {
            error!("Error accepting connection: {}", err);
            return Ok(());
        }
    };
    while let Some(msg) = ws.next().await {
        if let Ok(Message::Text(text)) = msg {
            if let Err(err) = handle_message(&text, enigo.clone(), stop_move_flag.clone()) {
                error!("Error handling message: {}", err);
            }
        }
    }

    if let Err(e) = client_disconnect_sender.send(client_id).await {
        error!("Failed to send client disconnect signal: {:?}", e);
    }
    info!("Client disconnected: {}", client_id);

    Ok(())
}

async fn handle_connection_http(
    stream: TcpStream,
    enigo: Arc<Mutex<Enigo>>,
    stop_move_flag: Arc<AtomicBool>,
    client_id: Uuid,
    client_disconnect_sender: mpsc::Sender<Uuid>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut ws = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(err) => {
            error!("Error accepting connection: {}", err);
            return Ok(());
        }
    };
    while let Some(msg) = ws.next().await {
        if let Ok(Message::Text(text)) = msg {
            if let Err(err) = handle_message(&text, enigo.clone(), stop_move_flag.clone()) {
                error!("Error handling message: {}", err);
            }
        }
    }

    if let Err(e) = client_disconnect_sender.send(client_id).await {
        error!("Failed to send client disconnect signal: {:?}", e);
    }
    info!("Client disconnected: {}", client_id);

    Ok(())
}

fn handle_message(
    text: &str,
    enigo: Arc<Mutex<Enigo>>,
    stop_move_flag: Arc<AtomicBool>,
) -> Result<(), JsonError> {
    let command = serde_json::from_str::<InputRequest>(text)?;
    let enigo_clone = enigo;
    let stop_flag_clone = stop_move_flag;
    thread::spawn(move || {
        let mut enigo = enigo_clone.lock().unwrap();
        handle_command(&mut *enigo, command, stop_flag_clone);
    });
    Ok(())
}

// Removed run_server as run function now handles loading certs and keys
// pub async fn run_server(
//     addr: &str,
//     cert_path: &str,
//     key_path: &str,
//     server: Arc<Mutex<Server>>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let certs = load_certs(cert_path)?;
//     let key = load_private_key(key_path)?;
//     let config = ServerConfig::builder()
//         .with_no_client_auth()
//         .with_single_cert(certs, key)?;
//     let config = Arc::new(config);

//     // (Unused) let connected_clients = server.lock().unwrap().connected_clients.clone();
//     // (Unused) let client_disconnect_sender = server.lock().unwrap().client_disconnect_sender.clone();
//     server.lock().unwrap().start(addr, config).await
// }

// Certificate and key loading helpers
fn load_certs(path: &str) -> Result<Vec<CertificateDer<'static>>, std::io::Error> {
    let mut reader = std::io::BufReader::new(File::open(path)?);
    rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
}

fn load_private_key(path: &str) -> Result<PrivateKeyDer<'static>, std::io::Error> {
    let mut reader = std::io::BufReader::new(File::open(path)?);
    let keys = rustls_pemfile::pkcs8_private_keys(&mut reader)
        .collect::<Result<Vec<_>, _>>()?;
    let pkcs8 = keys.into_iter().next().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "No private key found"))?;
    Ok(PrivateKeyDer::from(pkcs8))
}

fn handle_command(enigo: &mut Enigo, command: InputRequest, stop_move_flag: Arc<AtomicBool>) {
    match command {
        InputRequest::Mouse(request) => match request.command {
            MouseCommand::Move => {
                if stop_move_flag.load(Ordering::SeqCst) {
                    return;
                }
                crate::input::move_mouse(enigo, request.move_direction.x, request.move_direction.y);
                stop_move_flag.store(false, Ordering::SeqCst);
            }
            MouseCommand::Click => {
                crate::input::handle_mouse_action(enigo, &request);
            }
            MouseCommand::Scroll => match request.scroll.direction {
                ScrollDirection::X => crate::input::scroll_mouse_x(enigo, request.scroll.delta),
                ScrollDirection::Y => crate::input::scroll_mouse_y(enigo, request.scroll.delta),
            },
            MouseCommand::StopMove => {
                stop_move_flag.store(true, Ordering::SeqCst);
            }
        },
        InputRequest::Keyboard(request) => {
            crate::input::press_keys(enigo, &request);
        }
    }
}
