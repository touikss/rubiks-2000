extern crate gtk;

use gtk::{gdk_pixbuf, prelude::*};
use gtk::{Image, Window, WindowType};
use rscam::{Camera, Config};

mod imagery;

const CAMERA_WIDTH: u32 = 640;
const CAMERA_HEIGHT: u32 = 480;

fn main() {
    // Initialize GTK application
    gtk::init().expect("Failed to initialize GTK.");

    // Create a new window
    let window = Window::new(WindowType::Toplevel);
    window.set_title("Camera Viewer");
    window.set_default_size(CAMERA_WIDTH as i32, CAMERA_HEIGHT as i32);

    
    // Create an image widget
    let image = Image::new();
    window.add(&image);

    // Show all widgets
    window.show_all();

    // Connect signal to close the window when the 'X' button is clicked
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        gtk::glib::Propagation::Stop
    });

    // Open camera
    let mut camera = Camera::new("/dev/video0").unwrap();
    camera
        .start(&Config {
            interval: (1, 30),
            resolution: (CAMERA_WIDTH, CAMERA_HEIGHT),
            format: b"RGB3",
            ..Default::default()
        })
        .unwrap();

    // Start loop for capturing and displaying images
    loop {
        let frame = camera.capture().unwrap();

        let mut img = imagery::image::Image::new_frome_frame(&frame);

        let length = 50;
        let margin = 20;
        let delta = margin + length;

        let start_x = (CAMERA_WIDTH as f32 / 2. - length as f32 * 3. / 2. - margin as f32) as u32;
        let start_y = (CAMERA_HEIGHT as f32 / 2. - length as f32 * 3. / 2. - margin as f32) as u32;

        //img.apply_gray_filter();
        //img.apply_blur(1);
        //img.reduce();
        img.apply_mass_detection();

        for i in 0..3 {
            for j in 0..3 {
                //img.draw_square(start_x + i * delta, start_y + j * delta, length);
            }
        }
        
        let glib_bytes = gtk::glib::Bytes::from(img.bytes());

        let pixbuf = gdk_pixbuf::Pixbuf::from_bytes(
            &glib_bytes,
            gdk_pixbuf::Colorspace::Rgb,
            false,
            8,
            img.width() as i32,
            img.height() as i32,
            img.width() as i32 * 3,
        );

        image.set_from_pixbuf(Some(&pixbuf));

        // Process GTK events
        while gtk::events_pending() {
            gtk::main_iteration();
        }
    }
}
