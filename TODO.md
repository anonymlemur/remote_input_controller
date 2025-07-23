# 📝 Build & Event Loop Integration Log

## Completed Tasks
 - [x] Remove debug log for 'No ServerStatus received in this event loop iteration' in event loop
## Current Tasks

## Windows-Specific TODOs
- [x] Handle Windows-specific file paths
- [x] Test Windows system tray integration
- [ ] Add Windows firewall exception handling
- [ ] Test Windows UAC elevation requirements
- [ ] Verify Windows input simulation accuracy
- [ ] Add Windows startup registration option
- [ ] Improve Windows status window UI

## Remaining Tasks
### Tray Icon and Menu
- [x] Implement logic for showing/hiding menu items
- [x] Create basic status window
- [ ] Improve status window UI (replace msg.exe with proper window)
- [ ] Add QR code generation
    - [ ] Generate QR code for server address (wss://...)
    - [ ] Display QR code in GUI window
- [ ] Add server status display with refresh button

### Server Logic
- [x] Send server status updates (Started/Stopped) from server
- [x] Send client connected/disconnected count from server
- [ ] Add configurable server address/port
- [ ] Implement client authentication
- [ ] Add connection logging
- [ ] Improve error handling
- [ ] Improve SSL certificate handling
    - [ ] Allow user to provide custom SSL cert/key
    - [ ] Generate self-signed cert if missing
    - [ ] Add UI for SSL certificate status

### GUI Window
- [ ] Implement proper native Windows dialog for status
- [ ] Add server status display with auto-refresh
- [ ] Add QR code display (show QR for server address)
- [ ] Add connected clients list with management options
- [ ] Add configuration panel
- [ ] Add SSL certificate status/management UI

### Other Logic
- [ ] Implement Connect functionality
- [ ] Add settings configuration
- [ ] Add logging to file
- [ ] Create installer package
