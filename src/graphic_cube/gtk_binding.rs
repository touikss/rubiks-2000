//! This module contains the binding between **kiss3d** and the **gtk** ui.

use gtk::prelude::ImageExt;
use kiss3d::window::Window;

/// Draw the content of the **kiss3d** render on a **gtk image**.
/// 
/// ## Args
/// 
/// * `window` - the **kiss3d** window
/// * `gtk_image` - the gtk image to draw the render on 
/// 
/// ## Notes
/// 
/// It has to be called in every loop you call window.render() ( so every frame basically)
pub fn update_gtk_image(window: &mut Window, gtk_image: &mut gtk::Image) {
    
    // snap the kidd3D render
    let snaped = window.snap_image().into_vec();
    // convert it into bytes
    let bytes = gtk::glib::Bytes::from(&snaped);

    // create a pixel buffer from it
    let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_bytes(
        &bytes,
        gtk::gdk_pixbuf::Colorspace::Rgb,
        false,
        8,
        window.width() as i32,
        window.height() as i32,
        window.width() as i32 * 3,
    );

    // set the image from the pixel buffer
    (*gtk_image).set_from_pixbuf(Some(&pixbuf));

    // continue to treat the gtk events (otherwise ui will bug)
    while gtk::events_pending() {
        gtk::main_iteration();
    }
}
