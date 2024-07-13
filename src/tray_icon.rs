use fltk::{app, prelude::*, window::Window, image::RgbImage};

use std::result::Result;
use systray::{Application, Error};
use qrcode::QrCode;
use image::{ImageBuffer, Luma};

use crate::process;

pub fn create_tray_icon() -> Result<(), Error> {
    let mut app = Application::new()?;
    app.set_icon_from_file("E:/VS/web_socket_server/web_socket_server/src/icon.ico")?;
    app.add_menu_item("Enable", |_| {
        println!("Server started");
        //TODO: Start the server
        Ok::<_, systray::Error>(())
    })?;
    app.add_menu_item("Disable", |_| {
        println!("Server stopped");
        //TODO: Stop the server
        Ok::<_, systray::Error>(())
    })?;

    app.add_menu_item("Connect", |_| {
        println!("QR menu opend");
        generate_qr_code();

        Ok::<_, Error>(())
    })?;

    app.add_menu_item("Manage Connections", |_| {
        println!("Connection Manager opened");
        //TODO: Manage connections
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



fn generate_qr_code() {
    //TODO: Generate QR from unique ID, one per session
    let app = app::App::default();
    
    let data = "https://example.com";
    let qr = QrCode::new(data).expect("Failed to create QR code");
    
    let qr_image = qr.render::<Luma<u8>>()
        .min_dimensions(300, 300)
        .quiet_zone(false) 
        .build(); 

    let rgb_image = ImageBuffer::from_fn(qr_image.width(), qr_image.height(), |x, y| {
        match qr_image.get_pixel(x, y).0[0] {
            0 => image::Rgb([0, 0, 0]), 
            _ => image::Rgb([255, 255, 255]), 
        }
    });

    let (img_width, img_height) = rgb_image.dimensions();
    let raw_image_data = rgb_image.into_raw();

    let window_width = 300;
    let window_height = 300;

    let img_x = (window_width - img_width as i32) / 2;
    let img_y = (window_height - img_height as i32) / 2;

    let mut win = Window::new(100, 100, window_width, window_height, "QR Code");
    let image = RgbImage::new(&raw_image_data, img_width as i32, img_height as i32, fltk::enums::ColorDepth::Rgb8).unwrap();
    win.set_color(fltk::enums::Color::White);

    let mut frame = fltk::frame::Frame::new(img_x, img_y, img_width as i32, img_height as i32, "");
    frame.set_image(Some(image));

    win.end();
    win.show();
    app.run().unwrap();
}

