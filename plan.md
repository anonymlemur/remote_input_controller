# Remote Input Controller Tray Menu Server Control Plan

## Notes
- User's server does not start when clicking the tray menu's Start Server item.
- `server_command_tx.send(ServerCommand::Start)` is called, but server does not start.
- After switching to `.blocking_send` and adding panic diagnostics, the server now starts and works as expected.
- User now requests: when server is running, show only Stop; when server is off, show only Start in the tray menu.
- Proper server control requires a background task running the server loop, listening for Start/Stop commands.
- The server logic in `web_socket.rs` must handle `ServerCommand::Start` and `ServerCommand::Stop` correctly.
- Tray menu dynamic visibility for Start/Stop is now implemented and working.
- Stop server functionality does not actually stop the server yet—needs investigation/fix.
- Investigation revealed oneshot shutdown is unreliable for async server shutdown; switching to tokio::sync::watch for robust shutdown signaling is best practice.

## Task List
- [x] Search codebase for `server_command_tx` usage and flow
- [x] Search for `tokio::spawn` to verify server task startup
- [x] Ensure server async task is started at program launch and always running
- [x] Ensure `Server::run` handles Start/Stop commands and controls the listener appropriately
- [x] Ensure tray menu event handler sends correct commands (`Start`, `Stop`) to server
- [x] Test that clicking Start/Stop in the tray menu starts/stops the server as expected
- [x] Update tray menu so only Start or Stop is visible/enabled according to server status
- [ ] Ensure Stop menu item actually stops the server when clicked
- [ ] Refactor server shutdown to use tokio::sync::watch for reliable async shutdown

## Current Goal
Refactor shutdown to use tokio::sync::watch