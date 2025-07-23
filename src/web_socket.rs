use enigo::{Enigo, Settings};
use futures_util::{stream::StreamExt, SinkExt};
use serde_json::Error as JsonError;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};
use tokio::{net::{TcpListener, TcpStream}, sync::{oneshot, mpsc}, task::JoinHandle};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message, WebSocketStream};
use tokio_rustls::{
    rustls::{
        server::NoClientAuth,
        ServerConfig,
        PrivateKey,
        Certificate,
    },
    TlsAcceptor,
};
use std::fs::File;
use std::io::BufReader;
use uuid::Uuid;

pub mod input_types;
use input_types::*;

// Define a type alias for the WebSocket stream
type TlsWebSocketStream = WebSocketStream<tokio_rustls::TlsStream<TcpStream>>;

struct Server {
    listener: Option<TcpListener>,
    shutdown_sender: Option<oneshot::Sender<()>>,
    connected_clients: Arc<Mutex<HashMap<Uuid, TlsWebSocketStream>>>, // Use HashMap to track clients with UUIDs
    client_disconnect_sender: mpsc::Sender<Uuid>, // Sender to notify when a client disconnects
}

impl Server {
    fn new(client_disconnect_sender: mpsc::Sender<Uuid>) -> Self {
        Server {
            listener: None,
            shutdown_sender: None,
            connected_clients: Arc::new(Mutex::new(HashMap::new())),
            client_disconnect_sender,
        }
    }

    async fn start(
        &mut self,
        addr: &str,
        config: Arc<ServerConfig>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        println!("Listening on: {}", addr);
        self.listener = Some(listener);
        let (tx, rx) = oneshot::channel();
        self.shutdown_sender = Some(tx);

        let acceptor = TlsAcceptor::from(config);
        let enigo = Arc::new(Mutex::new(Enigo::new(&Settings::default()).unwrap()));
        let stop_move_flag = Arc::new(AtomicBool::new(false));
        let connected_clients = Arc::clone(&self.connected_clients);
        let client_disconnect_sender = self.client_disconnect_sender.clone();

        let listener = self.listener.as_ref().unwrap();

        tokio::select! {
            res = async {
                while let Ok((stream, _)) = listener.accept().await {
                    let enigo_clone = enigo.clone();
                    let stop_move_flag_clone = stop_move_flag.clone();
                    let acceptor_clone = acceptor.clone();
                    let connected_clients_clone = connected_clients.clone();
                    let client_disconnect_sender_clone = client_disconnect_sender.clone();
                    tokio::spawn(async move {
                        let client_id = Uuid::new_v4(); // Generate a unique ID for each client
                        if let Err(e) = handle_connection(stream, enigo_clone, stop_move_flag_clone, acceptor_clone, connected_clients_clone, client_id, client_disconnect_sender_clone).await {
                            eprintln!("Error handling connection: {}", e);
                        }
                    });
                }
                Ok::<(), Box<dyn std::error::Error>>(()) // Add this line to specify the return type
            } => res,
            _ = rx => {
                println!("Server shutting down");
                self.disconnect_clients().await.unwrap_or_else(|e| eprintln!("Error disconnecting clients: {:?}", e));
                Ok(()) // Add this line for the shutdown case
            }
        }
    }

    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(tx) = self.shutdown_sender.take() {
            tx.send(()).map_err(|_| "Failed to send shutdown signal")?;
        }
        Ok(())
    }

    pub async fn disconnect_clients(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut clients = self.connected_clients.lock().unwrap();
        for (_, client) in clients.iter_mut() {
            client.close(None).await?;
        }
        clients.clear();
        Ok(())
    }

    pub fn get_connected_clients_count(&self) -> usize {
        self.connected_clients.lock().unwrap().len()
    }
}

async fn handle_connection(
    stream: TcpStream,
    enigo: Arc<Mutex<Enigo>>,
    stop_move_flag: Arc<AtomicBool>,
    acceptor: TlsAcceptor,
    connected_clients: Arc<Mutex<HashMap<Uuid, TlsWebSocketStream>>>,
    client_id: Uuid,
    client_disconnect_sender: mpsc::Sender<Uuid>,
) -> Result<(), Box<dyn std::error::Error>> {
    let stream = acceptor.accept(stream).await?;
    let mut websocket = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(err) => {
            eprintln!("Error accepting connection: {}", err);
            return Ok(());
        }
    };

    // Add the new client to the map
    connected_clients.lock().unwrap().insert(client_id, websocket);

    while let Some(msg) = connected_clients.lock().unwrap().get_mut(&client_id).unwrap().next().await {
        if let Ok(Message::Text(text)) = msg {
            if let Err(err) = handle_message(&text, enigo.clone(), stop_move_flag.clone()) {
                eprintln!("Error handling message: {}", err);
            }
        }
    }

    // Remove the client from the map when the connection is closed
    connected_clients.lock().unwrap().remove(&client_id);
    if let Err(e) = client_disconnect_sender.send(client_id).await {
        eprintln!("Failed to send client disconnect signal: {:?}", e);
    }
    println!("Client disconnected: {}", client_id);

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

pub async fn run_server(
    addr: &str,
    cert_path: &str,
    key_path: &str,
    server: Arc<Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let certs = load_certs(cert_path)?;
    let key = load_private_key(key_path)?;
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    let config = Arc::new(config);

    let connected_clients = server.lock().unwrap().connected_clients.clone();
    let client_disconnect_sender = server.lock().unwrap().client_disconnect_sender.clone();
    server.lock().unwrap().start(addr, config).await
}

fn load_certs(path: &str) -> Result<Vec<Certificate>, Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(File::open(path)?);
    rustls_pemfile::certs(&mut reader)
        .map(|mut certs| certs.drain(..).map(Certificate).collect())
        .map_err(|_| "Invalid certificate file")
        .map_err(|e: &str| e.into())
}

fn load_private_key(path: &str) -> Result<PrivateKey, Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(File::open(path)?);
    rustls_pemfile::pkcs8_private_keys(&mut reader)
        .map(|mut keys| keys.remove(0).into()) // Assuming only one key in the file
        .map_err(|_| "Invalid private key file")
        .map_err(|e: &str| e.into())
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
