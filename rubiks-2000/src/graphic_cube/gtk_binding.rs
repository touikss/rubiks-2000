use gtk::prelude::ImageExt;
use kiss3d::window::Window;

/// This is the main function which handle binding between kiss3D
/// and the gtk image where we want to draw our 3D renderer
/// It has to be called in every loop you call window.render() ( so every frame basically)
pub fn update_gtk_image(window: &mut Window, gtk_image: &mut gtk::Image) {
    let snaped = window.snap_image().into_vec();
    let bytes = gtk::glib::Bytes::from(&snaped);

    let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_bytes(
        &bytes,
        gtk::gdk_pixbuf::Colorspace::Rgb,
        false,
        8,
        window.width() as i32,
        window.height() as i32,
        window.width() as i32 * 3,
    );
    while gtk::events_pending() {
        gtk::main_iteration();
    }

    (*gtk_image).set_from_pixbuf(Some(&pixbuf));
}