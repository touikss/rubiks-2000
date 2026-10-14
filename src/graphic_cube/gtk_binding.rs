//! This module contains the binding between **kiss3d** and the **gtk** ui.

use gtk::prelude::*;
use kiss3d::window::Window;

/// Set `pixbuf` on `gtk_image`, scaled to fit the space given by [`make_image_resizable`].
pub fn set_image_scaled(gtk_image: &gtk::Image, pixbuf: &gtk::gdk_pixbuf::Pixbuf) {
    // keep the full render to scale it again when the image is resized
    unsafe { gtk_image.set_data("full-render", pixbuf.clone()) };
    rescale_image(gtk_image);
}

fn rescale_image(gtk_image: &gtk::Image) {
    let Some(full) = (unsafe { gtk_image.data::<gtk::gdk_pixbuf::Pixbuf>("full-render") }) else {
        return;
    };
    let full = unsafe { full.as_ref() };
    let Some(area) = gtk_image.ancestor(gtk::ScrolledWindow::static_type()) else {
        return gtk_image.set_from_pixbuf(Some(full));
    };
    let (w, h) = (full.width() as f64, full.height() as f64);
    let ratio = (area.allocated_width() as f64 / w).min(area.allocated_height() as f64 / h);
    let scaled = full.scale_simple(
        ((w * ratio) as i32).max(1),
        ((h * ratio) as i32).max(1),
        gtk::gdk_pixbuf::InterpType::Bilinear
    );
    gtk_image.set_from_pixbuf(scaled.as_ref().or(Some(full)));
}

/// Wrap `gtk_image` in a `ScrolledWindow` filling the view, so that the image size
/// follows the window instead of forcing it to the render size.
pub fn make_image_resizable(gtk_image: &gtk::Image) {
    let Some(parent) = gtk_image.parent().and_then(|p| p.downcast::<gtk::Container>().ok()) else {
        return;
    };
    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::External)
        .vscrollbar_policy(gtk::PolicyType::External)
        .build();
    parent.remove(gtk_image);
    scrolled.add(gtk_image);
    parent.add(&scrolled);

    // make every container up to the view expand and fill
    let mut child: gtk::Widget = scrolled.clone().upcast();
    while let Some(anc) = child.parent() {
        child.set_hexpand(true);
        child.set_vexpand(true);
        child.set_halign(gtk::Align::Fill);
        child.set_valign(gtk::Align::Fill);
        if anc.is::<gtk::Stack>() || anc.is::<gtk::Window>() || anc.is::<gtk::Viewport>() {
            break;
        }
        if anc.type_().name() == "GtkAlignment" {
            anc.set_property("xscale", 1.0f32);
            anc.set_property("yscale", 1.0f32);
        } else if let Some(b) = anc.downcast_ref::<gtk::Box>() {
            let (_, _, padding, pack) = b.query_child_packing(&child);
            b.set_child_packing(&child, true, true, padding, pack);
        }
        child = anc;
    }

    let image = gtk_image.clone();
    scrolled.connect_size_allocate(move |_, _| rescale_image(&image));
    scrolled.show_all();
}

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
    set_image_scaled(gtk_image, &pixbuf);

    // continue to treat the gtk events (otherwise ui will bug)
    while gtk::events_pending() {
        gtk::main_iteration();
    }
}
