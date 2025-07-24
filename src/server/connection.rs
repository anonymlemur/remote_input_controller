// Connection handling module
use log::{info, error};
use enigo::Enigo;
use futures_util::stream::StreamExt;
use std::sync::{atomic::AtomicBool, Arc, Mutex};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use tokio_rustls::TlsAcceptor;
use uuid::Uuid;

use super::message_handler::handle_message;

pub async fn handle_connection(
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

pub async fn handle_connection_http(
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
