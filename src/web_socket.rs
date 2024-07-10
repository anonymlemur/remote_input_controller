use enigo::{Enigo, Settings};
use futures_util::stream::StreamExt;
use serde_json::Error as JsonError;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

pub mod input_types;
use input_types::*;

async fn handle_connection(
    stream: TcpStream,
    enigo: Arc<Mutex<Enigo>>,
    stop_move_flag: Arc<AtomicBool>,
) -> Result<(), JsonError> {
    let mut websocket = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(err) => {
            eprintln!("Error accepting connection: {}", err);
            return Ok(());
        }
    };

    while let Some(msg) = websocket.next().await {
        if let Ok(Message::Text(text)) = msg {
            if let Err(err) = handle_message(&text, enigo.clone(), stop_move_flag.clone()) {
                eprintln!("Error handling message: {}", err);
            }
        }
    }
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

pub async fn run_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on: {}", addr);
    let enigo = Arc::new(Mutex::new(Enigo::new(&Settings::default()).unwrap()));
    let stop_move_flag = Arc::new(AtomicBool::new(false));

    while let Ok((stream, _)) = listener.accept().await {
        let enigo_clone = enigo.clone();
        let stop_move_flag_clone = stop_move_flag.clone();
        if let Err(e) = handle_connection(stream, enigo_clone, stop_move_flag_clone).await {
            eprintln!("Error handling connection: {}", e);
        }
    }
    Ok(())
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
