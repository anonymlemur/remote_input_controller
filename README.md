# Remote Input Controller

## Overview
This project provides a remote input control system that enables users to send keyboard and mouse commands over a web socket connection. It's ideal for managing inputs on virtual machines or remote desktop environments.

## Features
- **Keyboard Input**: Support for various keyboard keys and modifiers.
- **Mouse Input**: Control mouse movements, clicks, and scrolling.
- **Web Socket Communication**: Real-time input command processing through web sockets.

## Installation
1. Clone the repository:
   ```bash
   git clone https://github.com/anonymlemur/web_socket_server
2. Install dependencies (assuming you are using Rust and Cargo):
   ```bash
   cargo build

## Usage
Start the server:
   ```bash
    cargo run
  ```
Connect to the server through a web socket client and send JSON formatted commands for keyboard and mouse control.


## Contributing

Contributions are welcome! Please fork the repository and open a pull request with your changes.
## License

This project is licensed under the MIT License - see the LICENSE.md file for details.
