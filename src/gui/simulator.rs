//! This module handle the **Simulator View**

// std
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant},
};

// local
use super::{
    menu::open_menu,
    render_data::{MoveType, RenderData},
    solution::{open_solution, Solution},
    utils::*,
    widget_pool::WidgetPool,
};
use crate::{
    critical,
    cube::{
        defs::*,
        enums::{ColorTag    , Move::*},
    },
    error,
    solver::solver_thread::solve_spec,
    warning, Parameters,
};

// others
use gtk::{
    ffi::{GtkStateFlags, GtkWidget},
    gdk::ffi::GdkRGBA,
    glib::translate::ToGlibPtr,
    prelude::*,
};
use kiss3d::{
    camera::Camera,
    nalgebra::{distance, Point3},
    ncollide3d::math::Point,
};

// extern c code from gtk to change the background color of a widget
// not avalaible in rust
extern "C" {
    fn gtk_widget_override_background_color(
        widget: *mut GtkWidget,
        state: GtkStateFlags,
        color: *const GdkRGBA,
    );
}

/// Open the simulator view
///
/// ## Args
///
/// * `widget_pool` - The widget pool
pub fn open_simulator(widget_pool: &WidgetPool) {
    // show the simulator view
    open_view(widget_pool, SIMULATOR_ID);
}

/// Initi the **Simulator View** by creating 3d renderer and connectin button signals.
///
/// ## Args
///
/// * `widget_pool` - the gtk widget pool
/// * `solution_arc` - the solution of the cube
///
/// ## Notes
///
/// * This function only have to be called **once**
/// * 3d rendering will be running in background to avoid multiple heavy initializations
/// * `solution_arc` is shared between **solution view** and **simulator view** to avoid multiple initializations
pub fn init(
    widget_pool: &WidgetPool,
    solution_arc: Arc<Mutex<Solution>>,
) -> Arc<Mutex<RenderData>> {
    // create render data struct
    let mut render_data = RenderData::new(widget_pool, SIM_3D_IMAGE);

    // first rendering to show the cube
    render_data.render();

    // arc and mutex are used to share our render data between thread used by buttons signals
    let rd_arc = Arc::new(Mutex::new(render_data));

    // connect mouse and button signals
    connect_mouse_signals(widget_pool, rd_arc.clone());
    connect_button_signals(widget_pool, rd_arc.clone(), solution_arc);

    // return it to send it to the camera view
    rd_arc
}

/// Connect all the button signals of the **Simulator View**
///
/// ## Args
///
/// * `wp` - the gtk widget pool
/// * `rd_arc` - the render data of the view
/// * `solution_arc` - the solution of the cube
fn connect_button_signals(
    wp: &WidgetPool,
    rd_arc: Arc<Mutex<RenderData>>,
    solution_arc: Arc<Mutex<Solution>>,
) {
    // --------------- OPEN ANIMATION ---------------

    // connected to the menu solver button
    // after clicked, display a cube animation
    let rd_clone = rd_arc.clone();
    let wp_clone = wp.clone();
    wp.get_button(MENU_SOLVER_BUTTON).connect_clicked(move |_| {
        // open the view
        super::simulator::open_simulator(&wp_clone);
        // handle event
        while gtk::events_pending() {
            gtk::main_iteration();
        }

        // display animation
        if let Some(mut rd) = rd_clone.try_lock().ok() {
            rd.render_animation();
        } else {
            error!("Cannot acces render data to play begining animation.");
        }
    });

    // --------------- MOVE BUTTONS SIGNALS ---------------

    // connect front button signal
    let rd_clone = rd_arc.clone();
    wp.get_button(FRONT_BUTTON).connect_clicked(move |_| {
        // if we can reach our render data, then render the move, else we do nothing
        if let Some(mut render_data) = rd_clone.try_lock().ok() {
            render_data.render_move(F1, MoveType::Simulator);
        } else {
            warning!("Unable to lock render data to render move : {}", F1);
        }
    });

    // connect f2 button signal
    let rd_clone = rd_arc.clone();
    wp.get_button(F2_BUTTON).connect_clicked(move |_| {
        // if we can reach our render data, then render the move, else we do nothing
        if let Some(mut render_data) = rd_clone.try_lock().ok() {
            render_data.render_move(F2, MoveType::Simulator);
        } else {
            warning!("Unable to lock render data to render move : {}", F2);
        }
    });

    // connect f3 button signal
    let rd_clone = rd_arc.clone();
    wp.get_button(F3_BUTTON).connect_clicked(move |_| {
        // if we can reach our render data, then render the move, else we do nothing
        if let Some(mut render_data) = rd_clone.try_lock().ok() {
            render_data.render_move(F3, MoveType::Simulator);
        } else {
            warning!("Unable to lock render data to render move : {}", F3);
        }
    });

    // connect back button signal
    let rd_clone = rd_arc.clone();
    wp.get_button(BACK_BUTTON).connect_clicked(move |_| {
        if let Some(mut render_data) = rd_clone.try_lock().ok() {
            render_data.render_move(B1, MoveType::Simulator);
        } else {
            warning!("Unable to lock render data to render move : {}", B1);
        }
    });

    // connect b2 button signal
    let rd_clone = rd_arc.clone();
    wp.get_button(B2_BUTTON).connect_clicked(move |_| {
        if let Some(mut render_data) = rd_clone.try_lock().ok() {
            render_data.render_move(B2, MoveType::Simulator);
        } else {
            warning!("Unable to lock render data to render move : {}", B2);
        }
    });

    // connect b3 button signal
    let rd_clone = rd_arc.clone();
    wp.get_button(B3_BUTTON).connect_clicked(move |_| {
        if let Some(mut render_data) = rd_clone.try_lock().ok() {
            render_data.render_move(B3, MoveType::Simulator);
        } else {
            warning!("Unable to lock render data to render move : {}", B3);
        }
    });

    // connect left button signal
    let rd_clone = rd_arc.clone();
    wp.get_button(LEFT_BUTTON).connect_clicked(move |_| {
        if let Some(mut render_data) = rd_clone.try_lock().ok() {
            render_data.render_move(L1, MoveType::Simulator);
        } else {
            warning!("Unable to lock render data to render move : {}", L1);
        }
    });

    // connect l2 button signal
    let rd_clone = rd_arc.clone();
    wp.get_button(L2_BUTTON).connect_clicked(move |_| {
        if let Some(mut render_data) = rd_clone.try_lock().ok() {
            render_data.render_move(L2, MoveType::Simulator);
        } else {
            warning!("Unable to lock render data to render move : {}", L2);
        }
    });

    // connect l3 button signal
    let rd_clone = rd_arc.clone();
    wp.get_button(L3_BUTTON).connect_clicked(move |_| {
        if let Some(mut render_data) = rd_clone.try_lock().ok() {
            render_data.render_move(L3, MoveType::Simulator);
        } else {
            warning!("Unable to lock render data to render move : {}", L3);
        }
    });

    // connect right button signal
    let rd_clone = rd_arc.clone();
    wp.get_button(RIGHT_BUTTON).connect_clicked(move |_| {
        if let Some(mut render_data) = rd_clone.try_lock().ok() {
            render_data.render_move(R1, MoveType::Simulator);
        } else {
            warning!("Unable to lock render data to render move : {}", R1);
        }
    });

    // connect r2 button signal
    let rd_clone = rd_arc.clone();
    wp.get_button(R2_BUTTON).connect_clicked(move |_| {
        if let Some(mut render_data) = rd_clone.try_lock().ok() {
            render_data.render_move(R2, MoveType::Simulator);
        } else {
            warning!("Unable to lock render data to render move : {}", R2);
        }
    });

    // connect r3 button signal
    let rd_clone = rd_arc.clone();
    wp.get_button(R3_BUTTON).connect_clicked(move |_| {
        if let Some(mut render_data) = rd_clone.try_lock().ok() {
            render_data.render_move(R3, MoveType::Simulator);
        } else {
            warning!("Unable to lock render data to render move : {}", R3);
        }
    });

    // connect up button signal
    let rd_clone = rd_arc.clone();
    wp.get_button(UP_BUTTON).connect_clicked(move |_| {
        if let Some(mut render_data) = rd_clone.try_lock().ok() {
            render_data.render_move(U1, MoveType::Simulator);
        } else {
            warning!("Unable to lock render data to render move : {}", U1);
        }
    });

    // connect u2 button signal
    let rd_clone = rd_arc.clone();
    wp.get_button(U2_BUTTON).connect_clicked(move |_| {
        if let Some(mut render_data) = rd_clone.try_lock().ok() {
            render_data.render_move(U2, MoveType::Simulator);
        } else {
            warning!("Unable to lock render data to render move : {}", U2);
        }
    });

    // connect u3 button signal
    let rd_clone = rd_arc.clone();
    wp.get_button(U3_BUTTON).connect_clicked(move |_| {
        if let Some(mut render_data) = rd_clone.try_lock().ok() {
            render_data.render_move(U3, MoveType::Simulator);
        } else {
            warning!("Unable to lock render data to render move : {}", U3);
        }
    });

    // connect down button signal
    let rd_clone = rd_arc.clone();
    wp.get_button(DOWN_BUTTON).connect_clicked(move |_| {
        if let Some(mut render_data) = rd_clone.try_lock().ok() {
            render_data.render_move(D1, MoveType::Simulator);
        } else {
            warning!("Unable to lock render data to render move : {}", D1);
        }
    });

    // connect d2 button signal
    let rd_clone = rd_arc.clone();
    wp.get_button(D2_BUTTON).connect_clicked(move |_| {
        if let Some(mut render_data) = rd_clone.try_lock().ok() {
            render_data.render_move(D2, MoveType::Simulator);
        } else {
            warning!("Unable to lock render data to render move : {}", D2);
        }
    });

    // connect d3 button signal
    let rd_clone = rd_arc.clone();
    wp.get_button(D3_BUTTON).connect_clicked(move |_| {
        if let Some(mut render_data) = rd_clone.try_lock().ok() {
            render_data.render_move(D3, MoveType::Simulator);
        } else {
            warning!("Unable to lock render data to render move : {}", D3);
        }
    });

    // --------------- OTHER BUTTONS SIGNALS ---------------

    // back button
    // go back to menu
    let wp_clone = wp.clone();
    wp.get_button(SIM_BACK_BUTTON).connect_clicked(move |_| {
        // open menu
        open_menu(&wp_clone)
    });

    // help button
    // no more usefull
    // let wp_clone = wp.clone();
    // wp.get_button(SIM_HELP_BUTTON).connect_clicked(move |_| {
    //     // open help
    //     open_help(&wp_clone)
    // });

    // solve button
    // open solution view, solve the cube and send the solution
    let wp_clone = wp.clone();
    let rd_clone = rd_arc.clone();
    wp.get_button(SIM_SOLUTION_BUTTON)
        .connect_clicked(move |_| {
            if let Some(rd) = rd_clone.try_lock().ok() {
                let cc = rd.gcube.face_cube;

                if cc.verify() == CUBE_OK {
                    // open solution view
                    open_solution(&wp_clone);

                    // get the current cube
                    let cube = rd.gcube.face_cube.clone();

                    // solve it
                    let sol = solve_spec(rd.gcube.face_cube, &wp_clone);

                    // send the solution
                    let solution_found = Solution::new(sol, cube.to_cubie_cube());
                    // if we can access solution render data
                    if let Some(mut solution) = solution_arc.try_lock().ok() {
                        // send it solution vec found
                        *solution = solution_found.clone();
                        solution.reset();
                        // open the view
                    } else {
                        warning!("Cannot send solution to solution view.");
                    }
                } else {
                    error!("Invalid cube provided to solver.")
                }
            }
        });

    // camera button
    // open camera view
    let wp_clone = wp.clone();
    wp.get_button(SIM_CAMERA_BUTTON).connect_clicked(move |_| {
        super::camera::open_camera(&wp_clone);
    });

    // reset button
    // reset the cube to default
    // reset the color table used
    let rd_clone = rd_arc.clone();
    wp.get_button(SIM_RESET_BUTTON).connect_clicked(move |_| {
        if let Some(mut rd) = rd_clone.try_lock().ok() {
            // reste gcube to default
            Parameters::set_use_defautl_color_table(true);

            //let f = rd.gcube.face_cube.reverse();

            //let c = rd.gcube.face_cube.to_cubie_cube();


            


            rd.gcube.set_face_cube(DEFAULT.to_facelet_cube());
            //rd.gcube.set_face_cube(f);
        } else {
            warning!("Cannot access render data to reset graphical cube.")
        }
    });

    // shuffle button
    // shuffle the cube
    let rd_clone = rd_arc.clone();
    wp.get_button(SIM_SHUFFLE_BUTTON).connect_clicked(move |_| {
        if let Some(mut rd) = rd_clone.try_lock().ok() {
            // shuffle cube
            rd.render_shuffle();
        } else {
            warning!("Cannot access render data to shuffle graphical cube.")
        }
    });

    // parameter button is handled in `parameter.rs`
}

/// Connect all the mouse signals of the **Simulator View**
///
/// ## Args
///
/// * `wp` - the gtk widget pool
/// * `rd_arc` - the render data of the view
fn connect_mouse_signals(wp: &WidgetPool, rd_arc: Arc<Mutex<RenderData>>) {
    // ---------------------------------------------
    // -------------- POP OVER SIGNALS -------------
    // ---------------------------------------------

    // cublet clicked that we want to change the color
    let prev_facelet_hovered: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));

    // if the user has clicked on a facelet or not
    let has_selected_facelet = Rc::new(RefCell::new(false));

    // get the main event box and the 3d event box
    let main_event_box: gtk::EventBox = wp.get(SIM_EVENT_BOX);
    let event_box_3d: gtk::EventBox = wp.get(SIM_3D_EVENT_BOX);

    // get the color selecting pop over
    let gtk_popover: gtk::Popover = wp.get(SIM_3D_POP_OVER);

    // attach it to the gtk image
    gtk_popover.set_relative_to(Some(&rd_arc.lock().unwrap().gtk_image));
    let custom_popover = CustomPopover::new(gtk_popover);
    let popover = Rc::new(RefCell::new(custom_popover));

    // get the button color grid
    let button_grid: gtk::Grid = wp.get(SIM_POP_OVER_GRID);

    // init each button
    for i in 0..3 {
        for j in 0..3 {
            // get button
            // we take it as a widget because of to_glib_none
            let widget: gtk::Widget = button_grid
                .child_at(i as i32, j as i32)
                .expect(&format!("Grid button not found at position {i:?}, {j:?}"));

            if let Some(color) = ColorTag::iter().nth((i * 3 + j) as usize) {
                let (r, g, b) = color.to_rgb_normalized();

                // create a gdk color
                let gdk_color = GdkRGBA {
                    red: r as f64,
                    green: g as f64,
                    blue: b as f64,
                    alpha: 1.0,
                };

                // get a pointer to our widget
                let button_widget_ptr: *mut GtkWidget = widget.to_glib_none().0;

                // use c code to override background color
                unsafe {
                    gtk_widget_override_background_color(
                        button_widget_ptr,
                        gtk_sys::GTK_STATE_FLAG_NORMAL,
                        &gdk_color,
                    );
                }

                // get the current button
                let button: gtk::Button = widget.downcast::<gtk::Button>().unwrap();

                // assign signal
                let popover_clone = popover.clone();
                let prev_clone = prev_facelet_hovered.clone();
                let has_selected_clone = has_selected_facelet.clone();
                let rd_clone = rd_arc.clone();
                button.connect_clicked(move |_| {
                    // if a facelet was hovered before

                    if let Some(mut rd) = rd_clone.try_lock().ok() {
                        if let Some(facelet) = *prev_clone.borrow() {
                            if let Some(facelet_node) = &mut rd.gcube.gfacelets[facelet] {
                                facelet_node.node.set_color(r, g, b);
                                rd.gcube.face_cube.set_facelet(facelet, color);
                            } else {
                                panic!(); //error!("Facelet {} of the cube is not defined.", facelet);
                            }
                        }
                        rd.render();
                    } else {
                        error!("Can not access render data to change cube color");
                    }

                    // hide pop over
                    popover_clone.borrow_mut().popdown();

                    *has_selected_clone.borrow_mut() = false;

                    // erase last hovered facelet
                    *prev_clone.borrow_mut() = None;
                });
            }
        }
    }

    // ---------------------------------------------
    // --------------- MOUSE SIGNALS ---------------
    // ---------------------------------------------

    // compute base values of the camera
    let (dist_x_z, dist_y_z, base_angle_x, base_angle_y, base_angle_z) =
        compute_camera_data(rd_arc.clone());

    let base_angle_x_rc = Rc::new(RefCell::new(base_angle_x));
    let base_angle_y_rc = Rc::new(RefCell::new(base_angle_y));
    let base_angle_z_rc = Rc::new(RefCell::new(base_angle_z));

    let is_clicked = Rc::new(RefCell::new(false));
    // position where drag started
    let drag_start_pos = Rc::new(RefCell::new((0.0, 0.0)));
    // instant when drag started
    let instant_clicked = Rc::new(RefCell::new(Instant::now()));

    // the delta color to apply on facelet hovered
    let delta_color = 0.8;

    // --------------- CLICK ---------------

    let drag_start_pos_clone = drag_start_pos.clone();
    let is_clicked_clone = is_clicked.clone();
    let instant_clicked_clone = instant_clicked.clone();

    main_event_box.connect_button_press_event(move |_, event| {
        // get clic position
        *drag_start_pos_clone.borrow_mut() = (event.root().0, event.root().1);

        // get current time
        *instant_clicked_clone.borrow_mut() = Instant::now();

        // set is clicked
        *is_clicked_clone.borrow_mut() = true;

        // propagate
        gtk::glib::Propagation::Proceed
    });

    // --------------- MOVMENT ---------------

    let rd_clone = rd_arc.clone();
    let base_a_x_clone = base_angle_x_rc.clone();
    let base_a_y_clone = base_angle_y_rc.clone();
    let base_a_z_clone = base_angle_z_rc.clone();
    let is_clicked_clone = is_clicked.clone();

    main_event_box.connect_motion_notify_event(move |_, event| {
        if *is_clicked_clone.borrow() {
            if let Some(rd) = &mut rd_clone.try_lock().ok() {
                // get the camera
                let camera = &mut rd.camera;
                let mut eye = camera.eye();

                // compute dx and dy, deltas in pixels from starting point
                let (sx, sy) = *drag_start_pos.borrow();
                let dx = event.root().0 - sx;
                let dy = event.root().1 - sy;

                // compute new x, y and z position
                // for x and z: basic circle equation
                eye.x = dist_x_z * (*base_a_x_clone.borrow() - (dx as f32) / SENSITIVITY_X).cos();
                eye.z = dist_x_z * (*base_a_z_clone.borrow() - (dx as f32) / SENSITIVITY_X).sin();

                // for y, simple trigo but we clamp it
                let mut y_angle =
                    (*base_a_y_clone.borrow() + (dy as f32) / SENSITIVITY_Y).max(-1.0);
                y_angle = y_angle.min(1.0);
                eye.y = dist_y_z * y_angle.sin();

                // update camera position and make it look at the origin
                camera.look_at(eye, Point3::origin());

                // render
                rd.render();
            }
        }
        // propagate
        gtk::glib::Propagation::Proceed
    });

    // --------------- RELEASE ---------------
    let rd_clone = rd_arc.clone();
    let is_clicked_clone = is_clicked.clone();
    let instant_clicked_clone = instant_clicked.clone();
    let popover_clone = popover.clone();
    let has_selected_clone = has_selected_facelet.clone();
    let prev_clone = prev_facelet_hovered.clone();

    main_event_box.connect_button_release_event(move |_, _| {
        // set is clicked
        *is_clicked_clone.borrow_mut() = false;

        // if it was clicked not long ago
        if (*instant_clicked_clone.borrow()).elapsed().as_millis() < DELTA_TIME_CLICK {
            *has_selected_clone.borrow_mut() = false;

            // switch popover state
            if popover_clone.borrow().is_poped {
                popover_clone.borrow_mut().popdown();
            }

            if let Some(rd) = &mut rd_clone.try_lock().ok() {
                let mut prev_facelet = prev_clone.borrow_mut();
                if change_facelet_color(rd, *prev_facelet, -delta_color) {
                    *prev_facelet = None;
                }
                rd.render();
            } else {
                warning!("Cannot access render data when clicked.");
            }
        } else {
            //  it is dragging

            // try getting rendering datas
            if let Some(rd) = &mut rd_clone.try_lock().ok() {
                // get the camera
                let camera = &mut rd.camera;

                // check parameters
                if Parameters::instance().reset_camera_after_drag {
                    // reset position
                    camera.look_at(
                        Point::new(CAMERA_BASE.0, CAMERA_BASE.1, CAMERA_BASE.2),
                        Point3::origin(),
                    );
                }

                // set base position so if reset pos cam is false it is fluid
                *base_angle_x_rc.borrow_mut() = (camera.eye().x / dist_x_z).acos();
                *base_angle_z_rc.borrow_mut() = (camera.eye().z / dist_x_z).asin();
                *base_angle_y_rc.borrow_mut() = (camera.eye().y / dist_y_z).asin();

                //println!("{}, {}, {}",  *base_angle_x_rc.borrow(),  *base_angle_y_rc.borrow(),  *base_angle_z_rc.borrow());

                // render it
                rd.render();
            }
        }
        gtk::glib::Propagation::Proceed
    });

    // --------------- MOVMENT 3D BOX ---------------

    let has_selected_clone = has_selected_facelet.clone();
    let prev_clone = prev_facelet_hovered.clone();
    let is_clicked_clone = is_clicked.clone();
    let rd_clone = rd_arc.clone();

    event_box_3d.connect_motion_notify_event(move |_, event| {
        if !*is_clicked_clone.borrow() && !*has_selected_clone.borrow() {
            // check if we can access the render data
            // if not, it is probably just that we are already rendering move and so we do not want
            // to do ray casting
            if let Some(rd) = &mut rd_clone.try_lock().ok() {
                let (x, y) = event.root();

                // if it exists, reset his color
                let mut prev_facelet = prev_clone.borrow_mut();
                if change_facelet_color(rd, *prev_facelet, -delta_color) {
                    *prev_facelet = None;
                }

                // check for raycast
                if let Some(facelet) = rd.raycast(x, y) {
                    if change_facelet_color(rd, Some(facelet), delta_color) {
                        *prev_facelet = Some(facelet);
                    }
                }

                rd.render();
            } else {
                warning!("Cannot access render data when movment notified.");
            }
        }
        gtk::glib::Propagation::Proceed
    });

    // --------------- RELEASE 3D BOX ---------------
    let instant_clicked_clone = instant_clicked.clone();
    let rd_clone = rd_arc.clone();
    event_box_3d.connect_button_release_event(move |_, _| {
        // set is clicked
        *is_clicked.borrow_mut() = false;

        let mut prev_facelet = prev_facelet_hovered.borrow_mut();

        // if it was clicked not long ago
        if (*instant_clicked.borrow()).elapsed().as_millis() < DELTA_TIME_CLICK {
            let mut has_selected = has_selected_facelet.borrow_mut();
            let mut popover = popover.borrow_mut();

            if !popover.is_poped {
                if let Some(_) = *prev_facelet {
                    // switch popover state
                    popover.popup();
                    *has_selected = true;
                }
            } else {
                *has_selected = false;
                popover.popdown();
                if let Some(rd) = &mut rd_clone.try_lock().ok() {
                    if change_facelet_color(rd, *prev_facelet, -delta_color) {
                        *prev_facelet = None;
                    }
                } else {
                    warning!(
                        "Cannot access render data to change facelet color when released on 3d box."
                    );
                }
            }

            // prevent the main box to run a click event also
            *instant_clicked_clone.borrow_mut() -=
                Duration::from_millis((DELTA_TIME_CLICK as u64) + 10);
        }
        gtk::glib::Propagation::Proceed
    });
}

// compute base distance and angle of the camera
fn compute_camera_data(render_data_arc: Arc<Mutex<RenderData>>) -> (f32, f32, f32, f32, f32) {
    if let Some(mut rd) = render_data_arc.try_lock().ok() {
        // get the camera
        let camera = &mut rd.camera;
        let eye = camera.eye();
        camera.look_at(eye, Point3::origin());

        // get basic distances
        let dist_x_z = distance(&Point3::new(eye.x, 0.0, eye.z), &Point3::origin());
        let dist_y_z = distance(&Point3::new(0.0, eye.y, eye.z), &Point3::origin());

        // base angle of the camera for each axes (z = pi - x)get_button
        let base_angle_x: f32 = (eye.x / dist_x_z).acos();
        let base_angle_z: f32 = (eye.z / dist_x_z).asin();
        let base_angle_y: f32 = (eye.y / dist_y_z).asin();

        (dist_x_z, dist_y_z, base_angle_x, base_angle_y, base_angle_z)
    } else {
        critical!("Cannot access render data to compute camera datas");
    }
}

/// Change the color of a facelet depending on the delta color.
///
/// ## Args
///
/// * `render_data` - the rendering data of the view
/// * `facelet` - the facelet to change the color
/// * `delta_color` - the delta color to apply
///
/// ## Returns
///
/// * [true] if the facelet exists and was changed
/// * [false] otherwise
pub fn change_facelet_color(
    render_data: &mut MutexGuard<RenderData>,
    facelet_opt: Option<usize>,
    delta_color: f32,
) -> bool {
    if let Some(facelet) = facelet_opt {
        if let Some(facelet_node) = render_data
            .gcube
            .gfacelets
            .get_mut(facelet)
            .expect("FaceletNode not found")
        {
            let color = *facelet_node.node.data().object().unwrap().data().color();
            facelet_node.node.set_color(
                color.x + delta_color,
                color.y + delta_color,
                color.z + delta_color,
            );

            render_data.render();

            return true;
        } else {
            error!("Facelet is not defined at index: {}", facelet);
        }
    }
    false
}
