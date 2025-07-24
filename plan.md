# Remote Input Controller App Fix Plan

## Notes
- User encountered unresolved import `native_dialog::MessageType` error; verified correct usage for native-dialog 0.9.0.
- Cargo.toml dependency is correct; clean build attempted but errors persist.
- Implemented `generate_qr_svg` to create SVG and PNG QR codes from a string.
- Need to ensure QR code menu item in main.rs triggers QR code generation and display.
- QR code menu logic in main.rs updated to call generate_qr_svg and show_qr_svg_window with correct error handling.
- Removed all usage of MessageDialog and MessageType; updated to new native-dialog API.
- Fixed QR code PNG rendering to use qrcode::render::image::Luma and enabled image feature for qrcode in Cargo.toml.
- Fixed show_qr_svg_window call to pass the path argument.

## Task List
- [x] Diagnose unresolved import error for native-dialog
- [x] Verify and clean Cargo build
- [x] Implement `generate_qr_svg` function in qr_code.rs
- [x] Integrate QR code menu logic in main.rs (call generate_qr_svg and show_qr_svg_window when menu item clicked)
- [x] Remove all usage of MessageDialog and MessageType; update to new native-dialog API
- [x] Fix QR code PNG rendering to use qrcode::render::image::Luma and enable image feature for qrcode in Cargo.toml
- [x] Fix show_qr_svg_window call to pass the path argument
- [ ] Test full QR code flow from menu
- [ ] Handle errors and edge cases for QR code generation/display

## Current Goal
Test full QR code flow from menu