//! This module handle the **Timer View**

use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

use super::{utils::*, widget_pool::WidgetPool};
use gtk::{prelude::*, Label};

#[derive(Clone)]
/// Store timer data
/// 
/// ## Fields
/// 
/// * `start_time` - duration since the last start
/// * `is_running` - if the timer is running or not
struct Timer {
    /// Duration since the last start
    start_time: Rc<RefCell<Instant>>,
    /// If the timer is running or not
    is_running: Rc<RefCell<bool>>,
}

impl Timer {
    /// Create a new [`Timer`].
    /// 
    /// ## Returns
    ///
    /// The new [`Timer`].
    /// 
    /// ## Notes
    /// 
    /// The [`Timer`] is stopped so `is_running` is set to false.
    /// `start_time` is set to `Instant::now()` but it will never be read since it will be reset when timer will start.
    fn new() -> Self {
        Self {
            start_time: Rc::new(RefCell::new(Instant::now())),
            is_running: Rc::new(RefCell::new(false)),
        }
    }

    /// Switch the state of the [`Timer`].
    /// If it starts the timer, capture new start instant
    fn switch(&self) {
        let mut is_running = self.is_running.borrow_mut();
        let mut start_time = self.start_time.borrow_mut();
        // inverse is _running
        *is_running = !*is_running;
        // capture new start instant if it starts
        if *is_running {
            *start_time = Instant::now();
        }
    }

    /// Stop the [`Timer`] (just set `is_running` to false).
    fn stop(&self) {
        *self.is_running.borrow_mut() = false;
    }

    /// Get the time elapsed since last start time.
    /// 
    /// ## Returns
    /// 
    /// * [`Some`] (duration) since last start if it is running
    /// * [`None`] otherwise
    fn get_time(&self) -> Option<Duration> {
        if *self.is_running.borrow() {
            Some(self.start_time.borrow().elapsed())
        } else {
            None
        }
    }

    /// Returns `is_running`
    fn is_running(&self) -> bool {
        *self.is_running.borrow()
    }
}

/// Open the timer view.
/// 
/// ## Args
/// 
/// * `widget_pool` - the gtk widget pool
pub fn open_timer_view(widget_pool: &WidgetPool) {
    // show the timer view
    open_view(widget_pool, TIMER_ID);
}

/// Connect all the timer view signals.
/// 
/// ## Notes
/// 
/// This function is usually called only once.
pub fn connect_signals(widgets: &WidgetPool) {
    // create timer instance
    let timer = Timer::new();

    // get timer switch button
    let switch_button: gtk::Button = widgets.get_button(TIMER_SWITCH_BUTTON);

    // get timer_label
    let timer_label: gtk::Label = widgets.get(TIMER_LABEL);

    // get button child if it exists else panic
    if let Some(child) = switch_button.child() {
        // get button label from child
        let button_label = child
            .downcast::<Label>()
            .expect("no timer button label found");

        // connect signal to switch button
        // we use clone because it is moved and we want to re use it later
        let timer_clone = timer.clone();
        let label_clone = timer_label.clone();
        switch_button.connect_clicked(move |_| {
            // switch timer state
            timer_clone.switch();
            if timer_clone.is_running() {
                // update label
                button_label.set_label("Stop");
                // strat loop
                timer_loop(timer_clone.clone(), label_clone.clone());
            } else {
                // update label
                button_label.set_label("Start");
            }
        });
    } else {
        panic!("no child found from timer button");
    }

    // connect back signal
    let widgets_clone = widgets.clone();
    widgets
        .get_button(TIMER_BACK_BUTTON)
        .connect_clicked(move |_| {
            // stop the timer
            timer.stop();
            // reset label text
            timer_label.set_text(&"00:000");
            // show menu view
            open_view(&widgets_clone, MENU_ID);
        });
}

/// Start the displaying loop (getting time, displaying it).
/// 
/// ## Notes
/// * The loop break when `timer.is_running` is set to false
/// * `is_running` change with button connected signals
fn timer_loop(timer: Timer, label: Label) {
    // add closure to idle loop
    gtk::glib::idle_add_local(move || {
        // if we get some time (so it is running)
        if let Some(time) = timer.get_time() {
            // take seconds, millis
            let seconds = time.as_secs();
            let millis = time.subsec_millis();
            // format it
            let time_text = format!("{:02}:{:03}", seconds, millis);
            // and display it
            label.set_text(&time_text);
            gtk::glib::ControlFlow::Continue
        }
        // else it is broken so we stop the loop
        else {
            gtk::glib::ControlFlow::Break
        }
    });
}
