# Remote Input Controller - QR Code Display Issue

## Notes
- Application crashes when displaying QR code window due to lack of OpenGL 2.0+ support (eframe/egui_glow backend).
- Error: "egui_glow requires opengl 2.0+"; occurs on systems with only OpenGL 1.1 (GDI Generic).
- Core server and tray features work; only the GUI QR code/status windows are affected.
- Need to provide a fallback for QR code display on systems without OpenGL 2.0+.
- Compilation and fallback implementation for QR code display are now successful.

## Task List
- [x] Diagnose cause of QR code window crash (OpenGL 2.0+ requirement)
- [x] Investigate and implement fallback for QR code display (e.g. open PNG/SVG in default image viewer, or provide web-based display)
- [ ] Add error handling for OpenGL initialization failure (show user-friendly message)
- [ ] Update documentation and TODO.md to reflect workaround and requirements

## Current Goal
Add error handling and update documentation