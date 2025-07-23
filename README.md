# Remote Input Controller

## Overview
This project provides a remote input control system that enables users to send keyboard and mouse commands over a secure web socket connection. It runs as a system tray application, making it ideal for managing inputs on virtual machines or remote desktop environments.

## Features
- **System Tray Application**: Easily manage the server lifecycle from a system tray icon.
- **Secure WebSocket Communication**: Real-time input command processing over a secure (WSS) connection using SSL.
- **Keyboard Input**: Support for various keyboard keys and modifiers.
- **Mouse Input**: Control mouse movements, clicks, and scrolling.
- **Unique Device IDs**: QR codes generated for connection include a unique device ID (currently for identification, not authorization).

## Installation

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/anonymlemur/remote_input_controller.git
    cd remote_input_controller
    ```

2.  **Generate SSL Certificates:**
    This application uses SSL for secure communication. You need to generate a self-signed certificate and key pair in the project's root directory. You can use `openssl` for this:
    ```bash
    openssl req -x509 -nodes -newkey rsa:2048 -keyout key.pem -out cert.pem -days 365 -subj "/C=US/ST=State/L=City/O=Organization/OU=Unit/CN=localhost"
    ```
    Make sure `cert.pem` and `key.pem` are in the root directory of the project.

3.  **Build the project:**
    Assuming you have Rust and Cargo installed, build the project in release mode:
    ```bash
    cargo build --release
    ```
    The executable will be located in the `target/release/` directory.

## Usage

1.  **Run the tray application:**
    Navigate to the release directory in your terminal and run the executable:
    ```bash
    ./target/release/remote_input_controller
    ```
    The application will start and appear as an icon in your system tray.

2.  **Manage the server from the tray icon:**
    Click on the tray icon to access the menu with options:
    *   **Start Server**: Starts the secure WebSocket server.
    *   **Stop Server**: Stops the running server.
    *   **Status**: Displays the current server status (Running/Stopped) and the number of connected clients.
    *   **Connect**: Generates a QR code containing the server's connection information (IP address, port, unique device ID, and certificate fingerprint) and saves it as a PNG file in the current directory.
    *   **Disconnect**: Disconnects all currently connected clients.
    *   **Exit**: Closes the application.

3.  **Connect from a client device:**
    *   Click the "Connect" menu item on the tray application to generate the QR code.
    *   Locate the saved QR code PNG file (`qrcode_<device_id>.png`).
    *   Use a WebSocket client application on your remote device that can scan QR codes and connect to a secure WebSocket (WSS) endpoint. The connection string embedded in the QR code will contain all necessary information (WSS URL, port, unique device ID, and certificate fingerprint for verification).
    *   Your client application should use the provided certificate fingerprint to verify the server's identity during the SSL handshake.

## WebSocket API (for Client Implementation)

The server listens for incoming WebSocket connections (WSS). Once connected, clients can send JSON formatted commands for keyboard and mouse control.

### Endpoint

`wss://<server_ip_address>:<port>/<device_id>?cert_fingerprint=<fingerprint>`

*   `<server_ip_address>`: The IP address of the machine running the server.
*   `<port>`: The port the server is listening on (default is 9000).
*   `<device_id>`: A unique ID generated for each connection attempt (provided in the QR code). Currently for identification, not authorization.
*   `<fingerprint>`: The SHA256 fingerprint of the server's SSL certificate.

### Request Format

All requests should be sent as JSON objects over the WebSocket connection.

Refer to the `InputRequest` enum and related structures in the source code (`src/web_socket/input_types.rs`) for the exact structure of keyboard and mouse command JSON objects.

### Example Request (Mouse Move)

```json
{
  "action": "Mouse",
  "command": "Move",
  "move_direction": {
    "x": 100,
    "y": 200
  }
}
```

## Contributing

Contributions are welcome! Please fork the repository and open a pull request with your changes.

## License

This project is licensed under the Mozilla Public License Version 2.0 - see the LICENSE.md file for details.
