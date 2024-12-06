//! This module contains util functions for the ui and basic contants definitions.

use std::time::{ Duration, Instant };

use gtk::prelude::{ PopoverExt, WidgetExt };

use super::widget_pool::WidgetPool;

// -------------- VIEW IDs --------------
pub const TIMER_ID: &str = "timer_view";
pub const MENU_ID: &str = "menu_view";
pub const SIMULATOR_ID: &str = "main_view";
pub const CAMERA_ID: &str = "camera_view";
pub const SOLUTION_ID: &str = "solution_view";
pub const HELP_ID: &str = "help_view";
pub const PARAMETERS_ID: &str = "param_window";
pub const VIEWS_ID: [&str; 6] = [TIMER_ID, MENU_ID, SIMULATOR_ID, CAMERA_ID, SOLUTION_ID, HELP_ID];

// -------------- CAMERA --------------
// used to switch face during camera detection
pub const SWITCH_FACE_CAMERA: [usize; 6] = [0, 2, 5, 4, 1, 3];
pub const SENSITIVITY_X: f32 = 100.0;
pub const SENSITIVITY_Y: f32 = 80.0;

pub const CAMERA_WIDTH: u32 = 640;
pub const CAMERA_HEIGHT: u32 = 480; //640
pub const RENDER_RATE: u64 = 5; //500;
pub const CAMERA_BASE: (f32, f32, f32) = (-5.0 * 0.8, 5.0 * 0.8, -7.0 * 0.8);
//pub const CAMERA_BASE: (f32, f32, f32) = (-1. * 0.8, 2. * 0.8, 7. * 0.8);

/// The duration of a cube rotation
pub const ROTATION_DURATION_CAMERA: f32 = 1.0;
/// The duration of a cube rotation
pub const ROTATION_DURATION_ANIMATION: f32 = 2.0;

/// Time between click an release under which it is considered as a click.
pub const DELTA_TIME_CLICK: u128 = 200;

pub const MAIN_WINDOW: &str = "main_window";

// -------------- HELP VIEW --------------
pub const HELP_BACK_BUTTON: &str = "help_back_button";

// -------------- SIMULATOR VIEW --------------
pub const SIM_3D_IMAGE: &str = "main_3d_image";
pub const SIM_WINDOW: &str = "main_window";
pub const SIM_BACK_BUTTON: &str = "main_back_button";
pub const SIM_HELP_BUTTON: &str = "main_help_button";
pub const SIM_SOLUTION_BUTTON: &str = "main_solve_button";
pub const SIM_CAMERA_BUTTON: &str = "main_camera_button";
pub const SIM_RESET_BUTTON: &str = "main_reset_button";
pub const SIM_SHUFFLE_BUTTON: &str = "main_shuffle_button";
pub const SIM_PARAM_BUTTON: &str = "main_param_button";
pub const SIM_3D_EVENT_BOX: &str = "main_3d_event_box";
pub const SIM_3D_POP_OVER: &str = "main_3d_pop_over";
pub const SIM_POP_OVER_GRID: &str = "main_pop_over_grid";
pub const SIM_EVENT_BOX: &str = "main_event_box";

pub const SIM_MOVE_BOX_2: &str = "main_box_move_2";
pub const SIM_MOVE_BOX_3: &str = "main_box_move_3";

// moves button
pub const FRONT_BUTTON: &str = "main_f_button";
pub const BACK_BUTTON: &str = "main_b_button";
pub const LEFT_BUTTON: &str = "main_l_button";
pub const RIGHT_BUTTON: &str = "main_r_button";
pub const DOWN_BUTTON: &str = "main_d_button";
pub const UP_BUTTON: &str = "main_u_button";
pub const F2_BUTTON: &str = "main_f2_button";
pub const B2_BUTTON: &str = "main_b2_button";
pub const L2_BUTTON: &str = "main_l2_button";
pub const R2_BUTTON: &str = "main_r2_button";
pub const D2_BUTTON: &str = "main_d2_button";
pub const U2_BUTTON: &str = "main_u2_button";
pub const F3_BUTTON: &str = "main_f3_button";
pub const B3_BUTTON: &str = "main_b3_button";
pub const L3_BUTTON: &str = "main_l3_button";
pub const R3_BUTTON: &str = "main_r3_button";
pub const D3_BUTTON: &str = "main_d3_button";
pub const U3_BUTTON: &str = "main_u3_button";

// moves labels text
pub const MOVE_LABEL_TEXT_COMPLETE: [&str; 6] = ["Front", "Back", "Left", "Right", "Up", "Down"];
pub const MOVE_LABEL_TEXT_RESTRICTED: [&str; 6] = ["F1", "B1", "L1", "R1", "U1", "D1"];

// moves label
pub const MOVE_LABEL: [&str; 6] = [
    "main_f_label",
    "main_b_label",
    "main_l_label",
    "main_r_label",
    "main_u_label",
    "main_d_label",
];



// -------------- MENU VIEW --------------
pub const MENU_SITE_BUTTON: &str = "menu_site_button";
pub const MENU_QUIT_BUTTON: &str = "menu_quit_button";
pub const MENU_TIMER_BUTTON: &str = "menu_timer_button";
pub const MENU_SOLVER_BUTTON: &str = "menu_solver_button";

// -------------- TIMER VIEW --------------
pub const TIMER_SWITCH_BUTTON: &str = "timer_switch_button";
pub const TIMER_LABEL: &str = "timer_label";
pub const TIMER_BACK_BUTTON: &str = "timer_back_button";

// -------------- SOLUTION VIEW --------------
pub const SOL_PREV_BUTTON: &str = "solve_prev_button";
pub const SOL_NEXT_BUTTON: &str = "solve_next_button";
pub const SOL_RESTART_BUTTON: &str = "solution_restart_button";
pub const SOL_3D_IMAGE: &str = "solution_3d_image";
pub const SOL_BACK_BUTTON: &str = "solve_back_button";
pub const SOL_SCALE: &str = "solution_scale";
pub const SOL_FINISH_BUTTON: &str = "solution_finish_button";
pub const SOL_LABEL: &str = "solution_label";
pub const SOL_INFOS_WINDOW: &str = "solution_infos";
pub const SOL_THREAD_LABEL: &str = "sol_thread_label";
pub const SOL_LENGTH_LABEL: &str = "sol_length_label";
pub const SOL_SOL_LABEL: &str = "sol_sol_label";
pub const SOL_N_SOL_LABEL: &str = "sol_n_sol_label";
pub const SOL_ELLAPSED_LABEL: &str = "sol_ellapsed_label";

// -------------- PARAMETER VIEW --------------
pub const PARAM_BACK_BUTTON: &str = "param_back_button";
pub const PARAM_RESET_CAM_CHECK_BOX: &str = "param_reset_cam_check_box";
pub const PARAM_WINDOW: &str = "param_window";
pub const PARAM_VERBOSE_CHECK: &str = "param_verbose_check";
pub const PARAM_CAMERA_RESET_CHECK: &str = "param_camera_reset_check";
pub const PARAM_ADVANCED_CHECK: &str = "param_advanced_mode_check";
pub const PARAM_SENSITIVITY_DETECTION_SCALE: &str = "param_sensitivity_detection_scale";
pub const PARAM_PURGE_RATE_LABEL: &str = "param_purge_rate_label";
pub const PARAM_PURGE_THRESHOLD_LABEL: &str = "param_purge_threshold_label";
pub const PARAM_SELECT_CAMERA_BUTTON: &str = "param_select_camera_button";
pub const PARAM_SOLVER_TIMEOUT_SCALE: &str = "param_solver_timeout_scale";
pub const PARAM_SOLVER_MAX_LENGTH_SCALE: &str = "param_solver_max_length_scale";
pub const PARAM_SHUFFLE_LENGTH_SCALE: &str = "param_shuffle_length_scale";
pub const PARAM_SOL_SPEED_SCALE: &str = "param_sol_speed_scale";
pub const PARAM_SOL_INFO_CHECK: &str = "param_solver_info_check";

// -------------- CAMERA VIEW --------------
pub const CAMERA_CAPTURE_IMAGE: &str = "camera_capture_image";
pub const CAMERA_3D_IMAGE: &str = "camera_3d_image";
pub const CAMERA_BACK_BUTTON: &str = "camera_back_button";
pub const CAMERA_NEXT_BUTTON: &str = "camera_next_button";
pub const CAMERA_PROGRESS_BAR: &str = "camera_progress_bar";
pub const CAMERA_INFO_LABEL: &str = "camera_info_label";
pub const CAMERA_BOTTOM_BOX: &str = "camera_bottom_box";
pub const CAMERA_3D_EVENT_BOX: &str = "camera_3d_event_box";
pub const CAMERA_GLOBAL_EVENT_BOX: &str = "camera_global_event_box";
pub const CAMERA_POP_OVER_GRID: &str = "camera_pop_over_grid";
pub const CAMERA_3D_POP_OVER: &str = "camera_3d_pop_over";
pub const CAMERA_LIST_DIALOG: &str = "camera_list_dialog";
pub const CAMERA_LIST_BOX: &str = "camera_list_box";
pub const CAMERA_CANCEL_DIALOG_BUTTON: &str = "camera_cancel_dialog_button";

//pub const FRAME_RATE: u32 = 60;

pub const FRAME_INTERVAL: f32 = 1.0 / 80.0;
pub const FRAME_LIMIT: u64 = 80;

pub const INFO_STRINGS: [&str; 11] = [
    "Modify your capture (click on facelets you want to change)",
    "Turn your cube to the RIGHT and align it with the green squares",
    "Modify your capture (click on facelets you want to change)",
    "Turn your cube to the RIGHT and align it with the green squares",
    "Modify your capture (click on facelets you want to change)",
    "Turn your cube to the RIGHT and align it with the green squares",
    "Modify your capture (click on facelets you want to change)",
    "Turn your cube to the RIGHT, then UPWARDS and align it with the green squares",
    "Modify your capture (click on facelets you want to change)",
    "Turn your cube UPWARD TWICE and align it with the green squares",
    "Modify your capture (click on facelets you want to change)",
];

/// Open a gtk view.
///
/// ## Args
///
/// * `widget_pool` - the widget pool of the ui
/// * `view_id` - the id of the view to open
pub fn open_view(widget_pool: &WidgetPool, view_id: &str) {
    // hide the view
    for view in VIEWS_ID {
        let gtk_view: gtk::Viewport = widget_pool.get(view);
        if view != view_id {
            gtk_view.set_visible(false);
        } else {
            gtk_view.set_visible(true);
        }
    }
    // hide parameters
    let parameters: gtk::ApplicationWindow = widget_pool.get(PARAM_WINDOW);
    parameters.hide();

    let solution_infos: gtk::ApplicationWindow = widget_pool.get(SOL_INFOS_WINDOW);
    solution_infos.hide();
}

pub fn delay(seconds: f32) {
    let milliseconds = (seconds * 1000.0) as u64;
    let duration = Duration::from_millis(milliseconds);
    let now = Instant::now();
    let end = now + duration;

    while Instant::now() < end {}
}

/// A custom gtk popover with a bool to know its state.
///
/// ## Fields
///
/// * `gtk_popover` - the gtk widget popover
/// * `is_poped` - wether it is poped up or not
#[derive(Clone)]
pub struct CustomPopover {
    /// The gtk widget popover
    gtk_popover: gtk::Popover,
    /// Wether it is poped up or not
    pub is_poped: bool,
}

impl CustomPopover {
    /// Create a new [`CustomPopover`].
    ///
    /// ## Args
    ///
    /// * `gtk_popover` - the gtk widgets which will be custom
    ///
    /// ## Returns
    ///
    /// The new [`CustomPopover`].
    pub fn new(gtk_popover: gtk::Popover) -> Self {
        // ensure that the popover is poped down
        gtk_popover.popdown();
        Self {
            gtk_popover,
            is_poped: false,
        }
    }

    /// Pop up the [`CustomPopover`]
    pub fn popup(&mut self) {
        // check if it is not already poped up
        if !self.is_poped {
            self.gtk_popover.popup();
            self.is_poped = true;
        }
    }

    /// Pop down the [`CustomPopover`]
    pub fn popdown(&mut self) {
        // check if it is not already poped down
        if self.is_poped {
            self.gtk_popover.popdown();
            self.is_poped = false;
        }
    }
}
