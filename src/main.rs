pub mod input;
pub mod web_socket;
mod tray_icon;

use std::process;

#[tokio::main]
async fn main() {
    let address = "0.0.0.0:9000";

    std::thread::spawn(|| {
        if let Err(e) = crate::tray_icon::create_tray_icon() {
            eprintln!("Failed to create tray icon: {}", e);
        }
    });
    // Attempt to run the web socket server and handle potential errors
    if let Err(e) = web_socket::run_server(address).await {
        eprintln!("Failed to start the server: {:?}", e);
        process::exit(1);

    }
}
