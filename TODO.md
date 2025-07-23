# 📝 Build & Event Loop Integration Log

## Event Loop Closure Signature Attempts

- [x] Try winit event loop closure with 3 arguments (`event, event_loop_target, control_flow`) and set `*control_flow = ControlFlow::Wait;` as before.
  **Result:** FAILED, winit 0.29 expects 2 args
- [x] Restore 2-argument closure (`event, control_flow`) for winit 0.29 event loop.
  **Result:** FAILED, still E0308
- [x] Investigate winit 0.29 event loop closure signature: persistent E0308 error (`expected EventLoopWindowTarget, found ControlFlow`). Check docs for correct usage.
  **Result:** DONE, docs say closure should be `(event, &EventLoopWindowTarget<T>)`, but E0308 error persists
- [x] Fix winit event loop closure: use `(event, event_loop_window_target)` as closure args.
  **Result:** FAILED, E0308 error remains: expected EventLoopWindowTarget, found ControlFlow
- [x] Debug `*control_flow` assignment: why is it expecting `EventLoopWindowTarget`? Review event loop closure body and usage of `*control_flow`.
  **Result:** FOUND: In winit 0.29, must use `event_loop_window_target.set_control_flow` instead of `*control_flow = ...`
- [x] Fix code to use `event_loop_window_target.set_control_flow(ControlFlow::Wait)` in the event loop closure.
  **Result:** DONE, build error resolved
- [x] Investigate runtime crash: `CGSConnectionByID` assertion failure (macOS).
  **Result:** Resolved by initializing tray icon within the winit event loop.

---

# ✅ Resolved Issues

- `CGSConnectionByID` assertion failure on macOS.
- `ByteCountNotDivisibleBy4` error when loading icon by using `image` crate for decoding.

---

# 🚧 In Progress

- Implementing tray icon menu logic (Start, Stop, Status, Disconnect).
- Creating a GUI window for server status and QR code display.

---

#  Tasks

## Tray Icon and Menu

- [x] Implement logic for "Start Server" menu item to send a signal to the server thread to start.
- [x] Implement logic for "Stop Server" menu item to send a signal to the server thread to stop.
- [ ] Implement logic to show/hide "Start Server" and "Stop Server" menu items based on the server state.
- [x] Implement logic for "Disconnect" menu item to send a signal to the server thread to disconnect all clients.
- [ ] Implement logic for "Status" menu item to open the status GUI window.

## Server Logic (`src/web_socket.rs`)

- [x] Add a mechanism to communicate server state (running/stopped) and information (address, connected clients count) back to the main thread.
- [x] Implement the server start and stop functionality based on signals from the main thread.
- [x] Implement the disconnect all clients functionality based on a signal from the main thread.

## GUI Window

- [ ] Integrate a GUI library (e.g., `egui`) compatible with `winit`.
- [ ] Create a new window to display server status and QR code.
- [ ] Display server status (running/stopped), address, and connected clients count in the window.
- [ ] Generate and display a QR code for the server address using `qrcode-generator`.

## Other Logic

- [ ] Clarify and implement the functionality of the "Connect" menu item.
- [ ] Address remaining compiler warnings (unused imports, enums, variables, Result).

---

# ⚠️ Remaining Warnings (to be addressed)

## Current Warnings (as of July 2025)

**Unused imports**
- `TrayIcon` (`main.rs`)

**Unused variables**
- `server_address`, `key_path`, `cert_fingerprint_clone`, `tray_icon`, `server_state_clone`, `cert_fingerprint_clone2`, `client_disconnect_tx_clone` (`main.rs:24, 26, 69, 76, 80, 81, 82`)

**Unused enum/variant**
- `ServerCommand` (never used) (`main.rs:17`)
- `ServerState::Running` (never constructed) (`main.rs:137`)

**Variable does not need to be mutable**
- `menu` (`main.rs:50`)
- `server` (`main.rs:113`)

---
