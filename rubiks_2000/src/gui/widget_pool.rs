//! This module contains the [`WidgetPool`] definitions

use gtk::{
    glib::{IsA, Object},
    prelude::BuilderExtManual,
    Builder, Button,
};

use crate::critical;

/// A "pool" allowing to access easly each gtk widget of a ui.
///
/// ## Fields
/// 
/// * `builder` - the gtk builder used by the gtk ui
/// 
/// ## Note
///
/// This struct is used to reduce code length and increase lisibility
/// the code in the example is a pattern use frequently with gtk,
/// so it need to be clean to avoid giant function for connecting signals.
///
/// ## Example
///
/// ### * Before:  
/// 
/// ```
/// let solve_back_button: gtk::Button = builder.object("solve_back_button").expect("no back button found");
/// solve_back_button.connect_clicked(move |_| {...});
/// ```
///
/// ### * After:
///    
/// ```
/// widgets.get_button("main_b_button").connect_clicked(move |_| {...});
/// ```
///
#[derive(Clone)]
pub struct WidgetPool {
    /// The gtk builder used by the gtk ui
    pub(super) builder: Builder,
}

impl WidgetPool {
    /// Create a new [`WidgetPool`] from a gtk [`Builder`]
    /// 
    /// ## Returns
    ///
    /// The new [`WidgetPool`].
    pub fn new(builder: Builder) -> Self {
        Self { builder }
    }

    /// Get a widget of any type.
    /// 
    /// ## Args
    /// 
    /// * `widget_name` - the name of the widget you are trying to get 
    /// 
    /// ## Returns
    /// 
    /// The widget if it exists.
    /// 
    /// ## Panics
    ///
    /// If the widget does not exist.
    ///  
    /// ## Warning
    /// It must be used with type inference.
    /// 
    /// ## Example
    /// 
    /// ```
    /// let window: gtk::Window = widget_pool.get("main_window");
    /// ```
    pub fn get<T: IsA<Object>>(&self, widget_name: &str) -> T {
        let opt = self.builder
            .object(widget_name);

        if let Some(obj) = opt {
            return obj
        } else {
            critical!("No '{}' found in the widget pool", widget_name)
        }
    }

    /// Get a [`gtk::Button`], does not need type inference
    /// 
    /// ## Args
    /// 
    /// * `button_name` - the name of the button you are trying to get 
    /// 
    /// ## Returns
    /// 
    /// The button if it exists.
    /// 
    /// ## Panics
    ///
    /// If the button does not exist.
    ///  
    /// ## Example
    /// 
    /// ```
    /// widget_pool.get_button("main_b_button").connect_clicked(move |_| {...});
    /// ```
    /// 
    /// ## Note
    ///
    /// This function was created because in most of case, when we want a widget, it is a button.
    pub fn get_button(&self, button_name: &str) -> Button {
        let opt = self.builder
            .object(button_name);
        if let Some(obj) = opt {
            obj
        } else {
            critical!("No {} found in the builder", button_name)
        }
    }
}
