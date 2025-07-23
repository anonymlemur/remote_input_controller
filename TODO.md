
# TODO: Clean Up Remaining Warnings

All blocking build errors are fixed! Only non-blocking warnings remain:

---

## ⚠️ Current Warnings (as of July 2025)

**Unused imports**
- `io::BufReader` (`web_socket.rs`)
- `server::NoClientAuth` (`web_socket.rs`)
- `TrayIcon` (`main.rs`)

**Unused variables**
- `connected_clients`, `client_disconnect_sender` (`web_socket.rs:131, 132, 225, 226`)
- `ws_opt` (should not be mutable) (`web_socket.rs:163`)
- `keys` (should not be mutable) (`web_socket.rs:239`)
- `server_address`, `key_path`, `cert_fingerprint_clone`, `tray_icon`, `server_state_clone`, `cert_fingerprint_clone2`, `client_disconnect_tx_clone` (`main.rs:24, 26, 69, 76, 80, 81, 82`)

**Unused enum/variant**
- `ServerCommand` (never used) (`main.rs:17`)
- `ServerState::Running` (never constructed) (`main.rs:137`)

**Variable does not need to be mutable**
- `menu` (`main.rs:50`)
- `server` (`main.rs:113`)

---

## 💡 Recommendations

- Remove unused imports and variables for a warning-free build.
- Prefix variables with `_` if you want to keep them for future use.
- Remove unused enums/variants if not needed.
- Remove unnecessary `mut` keywords.
- Handle all `Result`s, especially for UI/menu code, to avoid silent errors.
- Remove dead code for maintainability.

---

> If you want a warning-free build, let me know and I will clean up all warnings automatically!