# Remote Input Controller | WIP

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

## WebSocket API Documentation

### Overview

This documentation provides a comprehensive guide to the WebSocket API for handling input requests. The API allows clients to send structured input commands for keyboard and mouse actions to a server, which processes these commands accordingly.

### WebSocket Endpoint

**WebSocket URL:** `ws://<your_server_address>:<port>`

### Request Format

All requests should be sent as JSON objects over the WebSocket connection.

### InputRequest Enum

The `InputRequest` enum defines two main types of input requests: `Keyboard` and `Mouse`.

```json
{
  "action": "Keyboard",
  "key": "a",
  "modifiers": {
    "alt": false,
    "ctrl": false,
    "meta": false,
    "shift": false
  }
}
```

```json
{
  "action": "Mouse",
  "command": "Move",
  "move_direction": {
    "x": 100,
    "y": 200
  },
  "click": {
    "button": "Left",
    "click_type": "Single"
  },
  "scroll": {
    "delta": 10,
    "direction": "Y"
  }
}
```

### KeyboardRequest Object

Handles keyboard input with optional modifiers.

#### Properties

- **key** (string): The key to be pressed.
- **modifiers** (Modifiers): An object specifying the state of modifier keys.

#### Example

```json
{
  "action": "Keyboard",
  "key": "a",
  "modifiers": {
    "alt": false,
    "ctrl": true,
    "meta": false,
    "shift": false
  }
}
```

### Modifiers Object

Specifies the state of keyboard modifier keys.

#### Properties

- **alt** (boolean): Indicates if the Alt key is pressed.
- **ctrl** (boolean): Indicates if the Ctrl key is pressed.
- **meta** (boolean): Indicates if the Meta (Windows/Command) key is pressed.
- **shift** (boolean): Indicates if the Shift key is pressed.

### MouseRequest Object

Handles various mouse actions including moving, clicking, and scrolling.

#### Properties

- **command** (MouseCommand): Specifies the type of mouse command.
- **move_direction** (MoveRequest): Specifies the coordinates for mouse movement.
- **click** (ClickRequest): Details for mouse click actions.
- **scroll** (ScrollRequest): Specifies the scroll direction and delta.

#### Example

```json
{
  "action": "Mouse",
  "command": "Click",
  "click": {
    "button": "Left",
    "click_type": "Single"
  }
}
```

### MouseCommand Enum

Defines the types of mouse commands.

#### Values

- **Move**: Moves the mouse cursor.
- **Click**: Performs a mouse click action.
- **Scroll**: Scrolls the mouse.
- **StopMove**: Stops mouse movement.

### MoveRequest Object

Defines the coordinates for mouse movement.

#### Properties

- **x** (integer): The x-coordinate for mouse movement.
- **y** (integer): The y-coordinate for mouse movement.

#### Example

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

### ClickRequest Object

Specifies details for mouse click actions.

#### Properties

- **button** (MouseButton): The mouse button to be clicked.
- **click_type** (ClickType): The type of click action (single or double).

#### Example

```json
{
  "action": "Mouse",
  "command": "Click",
  "click": {
    "button": "Left",
    "click_type": "Single"
  }
}
```

### MouseButton Enum

Defines the mouse buttons.

#### Values

- **Left**: Left mouse button.
- **Right**: Right mouse button.
- **Middle**: Middle mouse button.

### ClickType Enum

Defines the types of click actions.

#### Values

- **Single**: Single click.
- **Double**: Double click.

### ScrollRequest Object

Specifies the scroll direction and delta.

#### Properties

- **delta** (integer): The amount to scroll.
- **direction** (ScrollDirection): The direction of the scroll (X or Y).

#### Example

```json
{
  "action": "Mouse",
  "command": "Scroll",
  "scroll": {
    "delta": 10,
    "direction": "Y"
  }
}
```

### ScrollDirection Enum

Defines the scroll directions.

#### Values

- **X**: Horizontal scroll.
- **Y**: Vertical scroll.

### Example WebSocket Interaction

#### Connecting to the WebSocket

```javascript
const socket = new WebSocket('ws://<your_server_address>:<port>');

socket.onopen = function(event) {
    console.log('Connected to the WebSocket server.');
};

socket.onmessage = function(event) {
    const response = JSON.parse(event.data);
    console.log('Received response:', response);
};

socket.onerror = function(error) {
    console.log('WebSocket Error:', error);
};

socket.onclose = function(event) {
    console.log('Disconnected from the WebSocket server.');
};
```

#### Sending a Keyboard Request

```javascript
const keyboardRequest = {
    action: 'Keyboard',
    key: 'a',
    modifiers: {
        alt: false,
        ctrl: true,
        meta: false,
        shift: false
    }
};

socket.send(JSON.stringify(keyboardRequest));
```

#### Sending a Mouse Move Request

```javascript
const mouseMoveRequest = {
    action: 'Mouse',
    command: 'Move',
    move_direction: {
        x: 100,
        y: 200
    }
};

socket.send(JSON.stringify(mouseMoveRequest));
```

## Contributing

Contributions are welcome! Please fork the repository and open a pull request with your changes.

## License

This project is licensed under the Mozilla Public License Version 2.0 - see the LICENSE.md file for details.
