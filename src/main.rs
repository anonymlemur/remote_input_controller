
pub mod input;
pub mod web_socket;


#[tokio::main]
async fn main() {
    let address = "192.168.1.46:9000";
    web_socket::run_server(address).await;
}

