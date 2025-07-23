/// Display the QR code SVG in a native window (Windows only, using native-dialog)
pub fn show_qr_svg_window(path: &str) {
    #[cfg(target_os = "windows")]
    {
        use native_dialog::MessageDialog;
        use native_dialog::MessageType;
        use std::fs;
        if let Ok(svg) = fs::read_to_string(path) {
            MessageDialog::new()
                .set_type(MessageType::Info)
                .set_title("Scan QR Code")
                .set_text("Scan this QR code with your mobile device to connect.\n\n(If you do not see the image, open qr_code.svg manually.)")
                .show_alert().ok();
        }
    }
}
/// Open the generated QR SVG file using the default system viewer (Windows only)
pub fn open_qr_svg(path: &str) {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let _ = Command::new("cmd").args(["/C", "start", path]).status();
    }
}
// qr_code.rs
// QR code generation utility for remote_input_controller

use qrcode::QrCode;
use qrcode::render::svg;
use std::fs::File;
use std::io::Write;

/// Generate a QR code SVG for the given text and save to the specified file path.
pub fn generate_qr_svg(text: &str, path: &str) -> std::io::Result<()> {
    let code = QrCode::new(text).expect("Failed to create QR code");
    let image = code.render::<svg::Color>().min_dimensions(256, 256).build();
    let mut file = File::create(path)?;
    file.write_all(image.as_bytes())?;
    Ok(())
}
