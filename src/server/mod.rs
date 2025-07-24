// Server module - Core server functionality
pub mod connection;
pub mod tls;
pub mod message_handler;
pub mod input_types;

pub use connection::*;
pub use tls::*;


use log::{info, warn, error};
use enigo::{Enigo, Settings};
use std::{
    collections::HashMap,
    path::Path,
    sync::{
        atomic::AtomicBool,
        Arc, Mutex,
    },
};
use tokio::{net::TcpListener, sync::mpsc};
use tokio_rustls::rustls::ServerConfig;
use uuid::Uuid;

use crate::{ServerCommand, ServerStatus};

pub struct Server {
    listener: Option<TcpListener>,
    shutdown_tx: Option<tokio::sync::watch::Sender<bool>>,
    shutdown_rx: Option<tokio::sync::watch::Receiver<bool>>,
    pub connected_clients: Arc<Mutex<HashMap<Uuid, ()>>>,
    client_disconnect_sender: mpsc::Sender<Uuid>,
    enigo: Option<Arc<Mutex<Enigo>>>,
    stop_move_flag: Option<Arc<AtomicBool>>,
    config: Option<Arc<ServerConfig>>,
}

impl Server {
    pub fn new(client_disconnect_sender: mpsc::Sender<Uuid>) -> Self {
        Server {
            listener: None,
            shutdown_tx: None,
            shutdown_rx: None,
            connected_clients: Arc::new(Mutex::new(HashMap::new())),
            client_disconnect_sender,
            enigo: None,
            stop_move_flag: None,
            config: None,
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
            if running && self.listener.is_some() {
                info!("[Server::run] Server is running, polling for commands and connections...");
                // When running, poll both command_rx and shutdown/accept logic
                tokio::select! {
                    Some(command) = command_rx.recv() => {
                        info!("[Server::run] Received command while running: {:?}", command);
                        match command {
                            ServerCommand::Stop => {
                                info!("[Server::run] Processing ServerCommand::Stop (while running)");
                                self.stop()?;
                                status_tx.send(ServerStatus::Stopped).await?;
                                running = false;
                                info!("[Server::run] Server stopped successfully");
                            }
                            ServerCommand::DisconnectClients => {
                                info!("[Server::run] Disconnecting all clients...");
                                self.disconnect_clients().await?;
                                status_tx.send(ServerStatus::ClientDisconnected(0)).await.ok();
                            }
                            ServerCommand::Start => {
                                warn!("[Server::run] Server already running; ignoring Start");
                            }
                        }
                    }
                    // Accept/shutdown logic
                    result = self.accept_or_shutdown(&status_tx) => {
                        info!("[Server::run] accept_or_shutdown returned: {:?}", result);
                        match result {
                            Ok(stopped) => {
                                if stopped {
                                    running = false;
                                    info!("[Server::run] Server accept loop exited, stopped");
                                } else {
                                    info!("[Server::run] accept_or_shutdown returned false, continuing...");
                                }
                            }
                            Err(e) => {
                                error!("[Server::run] Accept/shutdown error: {}", e);
                                running = false;
                            }
                        }
                    }
                }
            } else {
                // Not running: only poll for commands
                if let Some(command) = command_rx.recv().await {
                    match command {
                        ServerCommand::Start => {
                            info!("[Server::run] Starting server...");
                            let status_tx_clone = status_tx.clone();
                            match self.start_http(addr, config.clone(), status_tx_clone).await {
                                Ok(_) => {
                                    running = true;
                                    info!("[Server::run] Server started successfully");
                                }
                                Err(e) => {
                                    error!("[Server::run] Error starting server: {}", e);
                                    println!("[Server::run] Error starting server: {}", e);
                                },
                            }
                        }
                        ServerCommand::Stop => {
                            info!("[Server::run] Stop received but server not running");
                        }
                        ServerCommand::DisconnectClients => {
                            info!("[Server::run] Disconnecting all clients (not running)");
                            self.disconnect_clients().await?;
                            status_tx.send(ServerStatus::ClientDisconnected(0)).await.ok();
                        }
                    }
                } else {
                    // Channel closed, exit loop
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn start(
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
        info!("[Server::start_http] Attempting to bind to {}...", addr);
        let listener = match TcpListener::bind(addr).await {
            Ok(l) => {
                info!("[Server::start_http] Successfully bound to {}", addr);
                l
            },
            Err(e) => {
                error!("[Server::start_http] Failed to bind to {}: {}", addr, e);
                println!("[Server::start_http] Failed to bind to {}: {}", addr, e);
                return Err(Box::new(e));
            }
        };
        self.listener = Some(listener);
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        log::info!("[Server::start_http] Created shutdown watch channel: tx: {:p}, rx: {:p}", &shutdown_tx, &shutdown_rx);
        self.shutdown_tx = Some(shutdown_tx);
        self.shutdown_rx = Some(shutdown_rx);

        let enigo = Arc::new(Mutex::new(Enigo::new(&Settings::default()).unwrap()));
        let stop_move_flag = Arc::new(AtomicBool::new(false));
        let _connected_clients = Arc::clone(&self.connected_clients);
        let _client_disconnect_sender = self.client_disconnect_sender.clone();
        let _status_tx_main = status_tx.clone();

        // Send started status after binding
        match status_tx.send(ServerStatus::Started(addr.parse()?)).await {
            Ok(_) => info!("[Server::start_http] Sent ServerStatus::Started for {}", addr),
            Err(e) => error!("[Server::start_http] Failed to send ServerStatus::Started: {}", e),
        }

        // Store references for the accept_or_shutdown method to use
        self.enigo = Some(enigo);
        self.stop_move_flag = Some(stop_move_flag);
        self.config = config;
        
        info!("[Server::start_http] Setup complete, returning to main loop");
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(tx) = self.shutdown_tx.take() {
            log::info!("[Server::stop] Sending shutdown signal via watch channel");
            let send_result = tx.send(true);
            log::info!("[Server::stop] Result of tx.send(true): {:?}", send_result);
            match send_result {
                Ok(_) => log::info!("Shutdown signal sent"),
                Err(e) => log::warn!("Failed to send shutdown signal: {:?}", e),
            }
        } else {
            log::warn!("No shutdown sender present");
        }
        if self.listener.is_some() {
            log::info!("Dropping listener");
            self.listener.take();
        }
        self.shutdown_rx = None;
        self.shutdown_tx = None;
        Ok(())
    }

    pub async fn disconnect_clients(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement client disconnection logic
        Ok(())
    }

    pub fn get_connected_clients_count(&self) -> usize {
        self.connected_clients.lock().unwrap().len()
    }

    // Accept/shutdown logic helper
    pub async fn accept_or_shutdown(&mut self, status_tx: &mpsc::Sender<ServerStatus>) -> Result<bool, Box<dyn std::error::Error>> {
        info!("[accept_or_shutdown] Called, checking shutdown_rx...");
        if let Some(ref mut shutdown_rx) = self.shutdown_rx {
            let mut shutdown_rx = shutdown_rx.clone();
            info!("[accept_or_shutdown] shutdown_rx exists, checking listener...");
            let listener = match self.listener.as_ref() {
                Some(l) => {
                    info!("[accept_or_shutdown] Listener exists, entering select loop");
                    l
                },
                None => {
                    info!("[accept_or_shutdown] No listener, sleeping and returning false");
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    return Ok::<bool, Box<dyn std::error::Error>>(false);
                }
            };
            info!("[accept_or_shutdown] About to enter tokio::select!");
            tokio::select! {
                accept_result = listener.accept() => {
                    info!("[accept_or_shutdown] listener.accept() triggered");
                    match accept_result {
                        Ok((stream, addr)) => {
                            info!("[accept_or_shutdown] Accepted connection from {}", addr);
                            
                            // Handle the connection using stored references
                            if let (Some(enigo), Some(stop_move_flag), Some(connected_clients)) = 
                                (self.enigo.as_ref(), self.stop_move_flag.as_ref(), Some(&self.connected_clients)) {
                                
                                let enigo_clone = enigo.clone();
                                let stop_move_flag_clone = stop_move_flag.clone();
                                let connected_clients_clone = connected_clients.clone();
                                let client_disconnect_sender_clone = self.client_disconnect_sender.clone();
                                let status_tx_clone = status_tx.clone();
                                let client_id = Uuid::new_v4();
                                let config_clone = self.config.clone();

                                // Insert client and send status update BEFORE spawning
                                {
                                    let mut clients = connected_clients_clone.lock().unwrap();
                                    clients.insert(client_id, ());
                                    let count = clients.len();
                                    status_tx_clone.send(ServerStatus::ClientConnected(count)).await.ok();
                                }

                                tokio::spawn(async move {
                                    if let Err(e) = if let Some(tls_config) = config_clone {
                                        // TLS connection
                                        let acceptor = tokio_rustls::TlsAcceptor::from(tls_config);
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
                            
                            Ok::<bool, Box<dyn std::error::Error>>(false)
                        }
                        Err(e) => {
                            error!("[accept_or_shutdown] Error accepting connection: {}", e);
                            Err(Box::new(e))
                        }
                    }
                }
                changed = shutdown_rx.changed() => {
                    info!("[accept_or_shutdown] shutdown_rx.changed() triggered");
                    match changed {
                        Ok(_) => {
                            let shutdown_value = *shutdown_rx.borrow();
                            info!("[accept_or_shutdown] Shutdown signal value: {}", shutdown_value);
                            if shutdown_value {
                                info!("[accept_or_shutdown] Shutdown signal is true, stopping server");
                                self.disconnect_clients().await?;
                                status_tx.send(ServerStatus::Stopped).await.ok();
                                self.listener = None;
                                self.shutdown_rx = None;
                                self.shutdown_tx = None;
                                return Ok(true);
                            } else {
                                info!("[accept_or_shutdown] Shutdown signal is false, continuing");
                            }
                            Ok::<bool, Box<dyn std::error::Error>>(false)
                        }
                        Err(e) => {
                            error!("[accept_or_shutdown] Error waiting for shutdown: {}", e);
                            Err(Box::new(e))
                        }
                    }
                }
            }
        } else {
            info!("[accept_or_shutdown] No shutdown_rx, sleeping and returning false");
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            Ok::<bool, Box<dyn std::error::Error>>(false)
        }
    }
}
