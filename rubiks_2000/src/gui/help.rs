//! This module handle the **Help View**

use gtk::prelude::*;

use super::{
    simulator::open_simulator,
    utils::{open_view, HELP_BACK_BUTTON, HELP_ID},
    widget_pool::WidgetPool,
};

/// Open the **Help View**.
/// 
/// ## Args
/// 
/// * `widget_pool` - the gtk widget pool
pub fn open_help(widget_pool: &WidgetPool) {
    open_view(widget_pool, HELP_ID);
}

/// Connect the button signals
/// 
/// ## Args
/// 
/// * `wp` - the gtk widget pool
pub fn connect_signals(wp: &WidgetPool) {
    // back button signal
    let wp_clone = wp.clone();
    wp
        .get_button(HELP_BACK_BUTTON)
        .connect_clicked(move |_| {
            // open simulator view
            open_simulator(&wp_clone)
        });
}
