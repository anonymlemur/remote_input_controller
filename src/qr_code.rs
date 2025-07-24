use eframe::{NativeOptions, egui};
use image::DynamicImage;
use qrcode::QrCode;
use qrcode::render::svg;
// use image::Luma; // Used in code
use log::{info, warn};
use resvg::tiny_skia::{self, Transform};
use resvg::usvg::{self, fontdb};
use std::{path::Path, sync::Arc, fs};

/// Generate a QR code from the given data and save as a PNG file.
/// Returns the path to the generated PNG file.
pub fn generate_qr_code(data: &str) -> Result<String, Box<dyn std::error::Error>> {
    let code = QrCode::new(data.as_bytes())?;
    let svg_str = code.render::<svg::Color>().min_dimensions(256, 256).build();

    // Load system fonts
    let mut font_db = fontdb::Database::new();
    font_db.load_system_fonts();

    // Parse the SVG data
    let tree = usvg::Tree::from_str(&svg_str, &usvg::Options::default(), &font_db)?;

    // Create a pixmap to render to
    let pixmap_size = tree.size().to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())
        .ok_or("Failed to create pixmap")?;

    // Render the SVG to the pixmap
    let transform = Transform::identity();
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Save the pixmap as a PNG
    let png_path = "qr_code.png";
    pixmap.save_png(png_path)?;

    Ok(png_path.to_string())
}

/// Generate a QR code from the given data and save as an SVG file.
///
/// # Arguments
/// * `data` - The data to encode in the QR code
///
/// # Returns
/// Path to the generated SVG file
pub fn generate_qr_svg(data: &str) -> Result<String, Box<dyn std::error::Error>> {
    let code = QrCode::new(data.as_bytes())?;
    
    let svg_data = code.render()
        .min_dimensions(200, 200)
        .max_dimensions(400, 400)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .build();
    
    let svg_path = "qr_code.svg";
    fs::write(svg_path, svg_data)?;
    
    Ok(svg_path.to_string())
}

/// Display the QR code PNG image in a native window using eframe/egui.
///
/// # Arguments
/// * `png_path` - Path to the QR code PNG file
/// * `server_addr` - The server address to display in the QR code window
pub fn show_qr_png_window(
    png_path: &str,
    server_addr: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(png_path);
    if !path.exists() {
        return Err(format!("Could not find QR code image at '{}'", png_path).into());
    }

    let img = image::open(path).map_err(|e| format!("Failed to load QR code image: {}", e))?;

    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 500.0])
            .with_title("Scan QR Code")
            .with_resizable(false),
        ..Default::default()
    };

    // Try to open with eframe, fall back to system viewer on OpenGL failure
    match eframe::run_native(
        "QR Code",
        native_options,
        Box::new(move |_cc| {
            Ok(Box::new(QrApp {
                img: Arc::new(img),
                server_addr: server_addr.to_string(),
                show_copy_button: true,
            }) as Box<dyn eframe::App>)
        }),
    ) {
        Ok(_) => {
            // Clean up the temporary QR code file
            let _ = std::fs::remove_file(png_path);
            Ok(())
        }
        Err(e) => {
            // OpenGL compatibility issue - fall back to system viewer
            warn!("OpenGL not supported, falling back to system viewer: {}", e);
            open_in_system_viewer(png_path)?;
            Ok(())
        }
    }
}

/// Open the QR code image in the system's default image viewer
///
/// # Arguments
/// * `path` - Path to the image file
pub fn open_in_system_viewer(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;
    
    let path = Path::new(path);
    if !path.exists() {
        return Err(format!("Image file not found: {}", path.display()).into());
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(&["/C", "start", "", path.to_str().unwrap()])
            .spawn()
            .map_err(|e| format!("Failed to open image: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open image: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open image: {}", e))?;
    }

    Ok(())
}

/// Display QR code using the best available method
///
/// # Arguments
/// * `server_addr` - The server address to encode in the QR code
pub fn display_qr_code(server_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let qr_data = format!("http://{}", server_addr);
    
    // Generate both PNG and SVG
    let png_path = generate_qr_code(&qr_data)?;
    let svg_path = generate_qr_svg(&qr_data)?;
    
    info!("Generated QR code files: {} and {}", png_path, svg_path);
    
    // Try to display using eframe, fall back to system viewer
    match show_qr_png_window(&png_path, server_addr) {
        Ok(_) => Ok(()),
        Err(e) => {
            warn!("Failed to show QR window: {}", e);
            info!("Opening QR code in system viewer instead");
            
            // Try PNG first, then SVG
            if let Err(_) = open_in_system_viewer(&png_path) {
                open_in_system_viewer(&svg_path)?;
            }
            
            Ok(())
        }
    }
}

/// Simple eframe App to display a QR code image with additional controls.
struct QrApp {
    img: Arc<DynamicImage>,
    server_addr: String,
    show_copy_button: bool,
}

impl eframe::App for QrApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let rgba_image = self.img.to_rgba8();
        let (width, height) = rgba_image.dimensions();
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [width as usize, height as usize],
            rgba_image.as_raw(),
        );
        let texture = ctx.load_texture("qr_code", color_image, egui::TextureOptions::default());
        let texture_handle = texture.clone();

        egui::CentralPanel::default().show(ctx, |ui| {
            // Main title
            ui.vertical_centered(|ui| {
                ui.heading("Remote Control QR Code");
                ui.add_space(10.0);

                // Server address
                ui.label(
                    egui::RichText::new(&self.server_addr)
                        .size(16.0)
                        .color(egui::Color32::from_rgb(0, 100, 200)),
                );

                ui.add_space(20.0);

                // QR Code image
                let max_size = 300.0;
                let size = egui::Vec2::splat(max_size);
                ui.image(egui::ImageSource::Texture(egui::load::SizedTexture::new(
                    texture_handle.id(),
                    size,
                )));

                ui.add_space(20.0);

                // Instructions
                ui.label(
                    egui::RichText::new("Scan this QR code with your device")
                        .text_style(egui::TextStyle::Body)
                        .italics(),
                );

                ui.add_space(10.0);

                // Copy button
                if self.show_copy_button && ui.button("Copy Server Address").clicked() {
                    ui.ctx().copy_text(self.server_addr.clone());
                    self.show_copy_button = false;
                    ctx.request_repaint_after(std::time::Duration::from_secs(1));
                    let ctx_clone = ctx.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        ctx_clone.request_repaint();
                    });
                } else if !self.show_copy_button {
                    ui.label(
                        egui::RichText::new("✓ Copied to clipboard!").color(egui::Color32::GREEN),
                    );
                }

                ui.add_space(10.0);

                // Close button
                if ui.button("Close").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });
    }
}
