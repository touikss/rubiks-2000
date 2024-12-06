//! This module handle the **Parameter Window**

// std
use std::fs;

// local
use crate::Parameters ;
use super::{ utils::*, widget_pool::WidgetPool };

// other
use gtk::{
    prelude::{ BoxExt, ButtonExt, ContainerExt, LabelExt, RangeExt, ToggleButtonExt, WidgetExt },
    Button,
};

/// Connect all the signals of parameter window
///
/// # Args
///
/// * `widget_pool` - The widget pool
pub fn connect_signals(widget_pool: &WidgetPool) {
    let window: gtk::ApplicationWindow = widget_pool.get(PARAM_WINDOW);
    widget_pool.get_button(SIM_PARAM_BUTTON).connect_clicked(move |_| {
        if window.is_visible() {
            window.hide();
        } else {
            window.show_all();
        }
    });

    // -------- GLOBAL --------

    // verbose signam
    let verbose_check: gtk::CheckButton = widget_pool.get(PARAM_VERBOSE_CHECK);
    verbose_check.connect_toggled(|check_button| {
        Parameters::instance().verbose = check_button.is_active();
    });

    // -------- SIMULATOR --------

    // advanced mode signal
    let advanced_check: gtk::CheckButton = widget_pool.get(PARAM_ADVANCED_CHECK);
    let wp_clone = widget_pool.clone();
    advanced_check.connect_toggled(move |check_button| {
        let is_active = check_button.is_active();
        Parameters::instance().advanced_mode = is_active;

        // set the text of the moves 1
        for (index, id) in MOVE_LABEL.iter().enumerate() {
            let label: gtk::Label = wp_clone.get(id);
            if is_active {
                label.set_text(MOVE_LABEL_TEXT_RESTRICTED[index]);
            } else {
                label.set_text(MOVE_LABEL_TEXT_COMPLETE[index]);
            }
        }

        // hide or show the extended move buttons
        let move_box_2: gtk::Box = wp_clone.get(SIM_MOVE_BOX_2);
        let move_box_3: gtk::Box = wp_clone.get(SIM_MOVE_BOX_3);
        move_box_2.set_visible(is_active);
        move_box_3.set_visible(is_active);
    });

    let shuffle_length_scale: gtk::Scale = widget_pool.get(PARAM_SHUFFLE_LENGTH_SCALE);
    shuffle_length_scale.set_value(10.0);
    shuffle_length_scale.connect_value_changed(move |scale| {
        if scale.value() == 0.0 {
            scale.set_value(1.0)
        } else {
            Parameters::set_shuffle_length(scale.value() as u32);
        }
    });

    // camera drag reset signal
    let camera_reset_check: gtk::CheckButton = widget_pool.get(PARAM_CAMERA_RESET_CHECK);
    camera_reset_check.connect_toggled(|check_button| {
        Parameters::instance().reset_camera_after_drag = check_button.is_active();
    });

    // -------- CUBE SOLVER --------

    let timeout_scale: gtk::Scale = widget_pool.get(PARAM_SOLVER_TIMEOUT_SCALE);
    timeout_scale.set_range(1., 10.);
    timeout_scale.connect_value_changed(move |scale| {
        Parameters::set_solver_timeout(scale.value());
    });
    timeout_scale.set_value(5.0);

    let max_moves_scale: gtk::Scale = widget_pool.get(PARAM_SOLVER_MAX_LENGTH_SCALE);
    max_moves_scale.set_range(1., 50.);
    max_moves_scale.set_value(20.0);
    max_moves_scale.connect_value_changed(move |scale| {
        Parameters::set_solver_max_moves(scale.value() as u32);
    });

    // -------- CUBE DETECTION --------

    let sensitivity_scale: gtk::Scale = widget_pool.get(PARAM_SENSITIVITY_DETECTION_SCALE);
    let purge_rate_label: gtk::Label = widget_pool.get(PARAM_PURGE_RATE_LABEL);
    let purge_threshold_label: gtk::Label = widget_pool.get(PARAM_PURGE_THRESHOLD_LABEL);
    sensitivity_scale.set_value(50.0);
    sensitivity_scale.connect_value_changed(move |scale| {
        Parameters::set_detection_sensitivity(scale.value());
        // update labels
        purge_rate_label.set_text(&format!("{}", Parameters::purge_rate()));
        purge_threshold_label.set_text(&format!("{}", Parameters::purge_threshold()));
    });

    let camera_dialog: gtk::Dialog = widget_pool.get(CAMERA_LIST_DIALOG);
    widget_pool.get_button(CAMERA_CANCEL_DIALOG_BUTTON).connect_clicked(move |_| {
        camera_dialog.hide();
    });

    let window: gtk::ApplicationWindow = widget_pool.get(PARAM_WINDOW);
    let wp_clone = widget_pool.clone();
    widget_pool.get_button(PARAM_SELECT_CAMERA_BUTTON).connect_clicked(move |_| {
        window.hide();
        open_camera_list_dialog(&wp_clone);
    });

    // ---------- SOLUTION ----------

    let sol_speed_scale: gtk::Scale = widget_pool.get(PARAM_SOL_SPEED_SCALE);
    sol_speed_scale.set_range(1., 20.);
    sol_speed_scale.connect_value_changed(move |scale| {
        Parameters::set_solution_speed((21 - scale.value() as u32) * 136);

        if scale.value() == 20. {
            Parameters::set_solution_speed(10);
            Parameters::set_solution_animation_time(0.3);
            Parameters::set_show_preview(false);
        } else {
            Parameters::set_show_preview(true);
        }

    });
    sol_speed_scale.set_value(10.);

    let param_info_check: gtk::CheckButton = widget_pool.get(PARAM_SOL_INFO_CHECK);
    param_info_check.connect_toggled(|check_button| {
        Parameters::set_show_infos(check_button.is_active());
    });


}

/// List all the possible camera avalaible on the device.
fn list_cameras() -> Vec<String> {
    let mut cameras = Vec::new();
    let video_devices = fs
        ::read_dir("/dev/")
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry|
            entry
                .file_name()
                .to_str()
                .map(|s| s.starts_with("video"))
                .unwrap_or(false)
        );

    for device in video_devices {
        let device_path = device.path();
        // check if it is a valid camera for rs cam
        if let Ok(_) = rscam::new(device_path.to_str().unwrap()) {
            cameras.push(device_path.to_str().unwrap().to_string());
        }
    }

    cameras
}

pub fn open_camera_list_dialog(wp: &WidgetPool) {
    let camera_dialog: gtk::Dialog = wp.get(CAMERA_LIST_DIALOG);
    camera_dialog.show_all();

    let camera_box: gtk::Box = wp.get(CAMERA_LIST_BOX);
    let camera_list = list_cameras();

    // clear previous child
    for child in camera_box.children() {
        camera_box.remove(&child);
    }

    for path in camera_list {
        let button = Button::with_label(&path);

        // Clone references for the closure
        let path = path.clone();
        let dialog_clone = camera_dialog.clone();

        // Connect the clicked signal for the button
        button.connect_clicked(move |_| {
            println!("Selected camera: {}", path);
            Parameters::instance().camera_path = path.to_string();
            dialog_clone.hide();
        });

        // Add the button to the camera box
        camera_box.pack_start(&button, false, false, 5);
    }

    // Show all buttons
    camera_box.show_all();
}
