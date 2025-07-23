
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
- [ ] Investigate runtime crash: `CGSConnectionByID` assertion failure (macOS).  
  **Next:** Possible cause: tray icon or winit event loop initialization order.

---

# ⚠️ Remaining Warnings

All blocking build errors are fixed! Only non-blocking warnings remain:

---

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

# ❗ Runtime Crash

## CGSConnectionByID assertion failure

You are seeing a runtime crash:

    Assertion failed: (CGAtomicGet(&is_initialized)), function CGSConnectionByID, file CGSConnection.mm, line 424.

This is a macOS system-level error, often related to GUI code running outside the main thread or before the app is fully initialized. It is likely caused by tray icon or GUI code being run from a background thread or in a non-standard way. See below for next steps.

---

# 💡 Recommendations

- Remove unused imports and variables for a warning-free build.
- Prefix variables with `_` if you want to keep them for future use.
- Remove unused enums/variants if not needed.
- Remove unnecessary `mut` keywords.
- Handle all `Result`s, especially for UI/menu code, to avoid silent errors.
- Remove dead code for maintainability.



# TODO: Clean Up Remaining Warnings

All blocking build errors are fixed! Only non-blocking warnings remain:

---

## ⚠️ Remaining Warnings

---


## ⚠️ Current Warnings (as of July 2025)

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

## ❗ Runtime Crash (CGSConnectionByID assertion failure)

You are seeing a runtime crash:

    Assertion failed: (CGAtomicGet(&is_initialized)), function CGSConnectionByID, file CGSConnection.mm, line 424.

This is a macOS system-level error, often related to GUI code running outside the main thread or before the app is fully initialized. It is likely caused by tray icon or GUI code being run from a background thread or in a non-standard way. See below for next steps.

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