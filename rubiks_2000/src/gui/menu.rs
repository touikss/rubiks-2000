//! This module handle the **Menu View**

use std::process::Command;

use super::{ timer, utils::*, widget_pool::WidgetPool };
use gtk::prelude::*;

/// Open the menu view.
///
/// ## Args
///
/// * `widget_pool` - the gtk widget pool
pub fn open_menu(widget_pool: &WidgetPool) {
    open_view(widget_pool, MENU_ID);
}

/// Connect all the signals of the menu view.
///
/// ## Args
///
/// * `wp` - the gtk widget pool
pub fn connect_signals(wp: &WidgetPool) {
    // link the big cube image with the openning site link
    wp.get_button(MENU_SITE_BUTTON).connect_clicked(move |_| {
        // open our website
        open_site();
    });

    wp.get_button(MENU_SITE_BUTTON).trigger_tooltip_query();

    // get the main_window
    let main_window: gtk::Window = wp.get("main_window");

    // menu quit button
    wp.get_button(MENU_QUIT_BUTTON).connect_clicked(move |_| {
        // close the main window
        main_window.close();
    });

    // get the menu timer button
    let wp_clone = wp.clone();
    // link it with the open timer func
    wp_clone.get_button(MENU_TIMER_BUTTON).connect_clicked(move |_| {
        // show timer view
        timer::open_timer_view(&wp_clone);
    });

    // get the menu solver button
    //let wp_clone = wp.clone();
    //wp_clone.get_button(MENU_SOLVER_BUTTON).connect_clicked(move |_| {
    //    // show main view
    //    super::simulator::open_simulator(&wp_clone);
    //});
}

/// Open our web site in the default browser.
fn open_site() {
    // locate html file
    let local_site_folder = "../site-web/index.html";
    // open it
    let _ = Command::new("xdg-open").arg(local_site_folder).output();
}
