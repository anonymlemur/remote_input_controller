use fltk::{app, prelude::*, window::Window, image::RgbImage, button::Button, enums::Align, prelude::*};

use std::result::Result;
use systray::{Application, Error};
use qrcode::QrCode;
use image::{GenericImageView, ImageBuffer, Luma};

use crate::process;

pub fn create_tray_icon() -> Result<(), Error> {
    let mut app = Application::new()?;
    app.set_icon_from_file("E:/VS/web_socket_server/web_socket_server/src/icon.ico")?;
    app.add_menu_item("Enable", |_| {
        println!("Server started");
        // Start the server
        Ok::<_, systray::Error>(())
    })?;
    app.add_menu_item("Disable", |_| {
        println!("Server stopped");
        // Stop the server
        Ok::<_, systray::Error>(())
    })?;

    app.add_menu_item("Connect", |_| {
        println!("QR menu opend");
        // Here you would open a settings window or a dialog
        // open settings window with 3 checkboxes one button to show qr code to connect to the server
        // Box 1 allow keyboard input
        // Box 2 allow mouse input
        // Box 3 allow multiple connections
        // Button 1 show qr code to connect to the server
        generate_qr_code();

        Ok::<_, Error>(())
    })?;

    app.add_menu_item("Manage Connections", |_| {
        println!("Connection Manager opened");
        // Here you can manage connections
        Ok::<_, systray::Error>(())
    })?;

    app.add_menu_separator()?;
    //TODO: thread exit instead of end process
    app.add_menu_item("Exit", |_| -> Result<(), systray::Error> {
        process::exit(0);

        Ok(())
    })?;

    app.wait_for_message()?;
    Ok(())
}


fn generate_qr_code2() {
    let app = app::App::default();
    let mut win = Window::new(100, 100, 400, 400, "QR Code");

    // Generate QR Code
    let data = "https://example.com"; // Data to encode in the QR code
    let qr = QrCode::new(data).unwrap();
    let qr_image = qr.render::<Luma<u8>>().build(); // Render the QR code to an image

    // Convert the image to RGB
    let rgb_image: ImageBuffer<image::Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(qr_image.width(), qr_image.height(), |x, y| {
        match qr_image.get_pixel(x, y).0[0] {
            0 => image::Rgb([0, 0, 0]),
            _ => image::Rgb([255, 255, 255]),
        }
    });

    // Convert the image buffer to a vector
    let (width, height) = rgb_image.dimensions();
    let raw_image_data = rgb_image.into_raw();

    // Create an FLTK image from the raw image data
    let image = RgbImage::new(&raw_image_data, width as i32, height as i32, fltk::enums::ColorDepth::Rgb8).unwrap();
    let mut frame = Button::new(100, 50, width as i32, height as i32, "");
    frame.set_image(Some(image));

    win.end();
    win.show();
    app.run().unwrap();


}


fn generate_qr_code() {
    let app = app::App::default();
    
    let data = "https://example.com";
    let qr = QrCode::new(data).expect("Failed to create QR code");
    
    // Render the QR code with no margin
    let qr_image = qr.render::<Luma<u8>>()
        .min_dimensions(300, 300)
        .quiet_zone(false)  // Assuming this removes the margin
        .build(); 

    // Convert QR code to RGB image
    let rgb_image = ImageBuffer::from_fn(qr_image.width(), qr_image.height(), |x, y| {
        match qr_image.get_pixel(x, y).0[0] {
            0 => image::Rgb([0, 0, 0]), // Black for QR code pixels
            _ => image::Rgb([255, 255, 255]), // White for background
        }
    });

    let (img_width, img_height) = rgb_image.dimensions();
    let raw_image_data = rgb_image.into_raw();

    // Define your desired window size
    let window_width = 300; // Example window width
    let window_height = 300; // Example window height

    // Center the image within the window
    let img_x = (window_width - img_width as i32) / 2;
    let img_y = (window_height - img_height as i32) / 2;

    let mut win = Window::new(100, 100, window_width, window_height, "QR Code");
    let image = RgbImage::new(&raw_image_data, img_width as i32, img_height as i32, fltk::enums::ColorDepth::Rgb8).unwrap();
    win.set_color(fltk::enums::Color::White);
    // Create a frame for the image at the calculated position
    let mut frame = fltk::frame::Frame::new(img_x, img_y, img_width as i32, img_height as i32, "");
    frame.set_image(Some(image));

    win.end();
    win.show();
    app.run().unwrap();
}

