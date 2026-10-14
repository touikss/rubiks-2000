//! Handle the **Camera View**

// std
use std::{
    cell::{RefCell, RefMut},
    f32::consts::{FRAC_PI_2, PI},
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant},
};

// gtk
use gtk::{
    gdk::{ffi::GdkRGBA, gdk_pixbuf},
    glib::{translate::ToGlibPtr, Cast, ObjectExt, Propagation, SignalHandlerId},
    prelude::*,
};

use gtk_sys::{gtk_widget_override_background_color, GtkWidget};

// kiss3d
use kiss3d::nalgebra::{Point3, UnitQuaternion};

// rscam
use rscam::{Camera, Config};

// local
use crate::{
    cube::{cubie_cube::CubieCube, enums::{ColorTag, Corner, Edge}, face_cube::FaceCube},
    error,
    imagery::{
        self,
        color::{ColorTable, RgbColor, RgbData},
        image::Image,
    },
    info, warning, Parameters,
};

use super::{
    render_data::RenderData,
    simulator::{change_facelet_color, open_simulator},
    utils::*,
    widget_pool::WidgetPool,
};

pub const CONV_MATRIX: [usize; 54] = [
    2, 5, 8, 1, 4, 7, 0, 3, 6, 15, 12, 9, 16, 13, 10, 17, 14, 11, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 51, 48, 45, 52, 49,
    46, 53, 50, 47,
];

pub const MATRIX: [usize; 54] = [
    6, 3, 0, 7, 4, 1, 8, 5, 2, 11, 14, 17, 10, 13, 16, 9, 12, 15, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 47, 50, 53, 46, 49,
    52, 45, 48, 51,
];

/// The order of the face detected by the cube
const FACE_ORDER: [usize; 6] = [
    ColorTag::F as usize,
    ColorTag::L as usize,
    ColorTag::U as usize,
    ColorTag::B as usize,
    ColorTag::R as usize,
    ColorTag::D as usize,
];

/// The rotation that cube will do for each step
const CUBE_MOVES: [(f32, f32, f32); 5] = [
    (0.0, FRAC_PI_2, 0.0), //  to go from face F to face L
    (0.0, 0.0, FRAC_PI_2), // ...
    (0.0, FRAC_PI_2, 0.0),
    (0.0, 0.0, FRAC_PI_2),
    (0.0, FRAC_PI_2, 0.0),
];

/// The camera capture width
const CAMERA_WIDTH: u32 = 640;
/// The camera capture height
const CAMERA_HEIGHT: u32 = 480;

/// The default color for colored buttons
const DEFAULT_BUTTON_COLOR: RgbColor = RgbColor::new(128, 128, 128);
/// The delta color to apply to hovered facelet
const DELTA_COLOR: f32 = 0.8;

/// Open the **Camera View**
///
/// ## Args
///
/// * `widget_pool` - the gtk widget pool
pub fn open_camera(widget_pool: &WidgetPool) {
    open_view(widget_pool, CAMERA_ID);
}

/// Initialize the camera 3d data and connect all signals.
///
/// ## Args
///
/// * `widget_pool` - the gtk widget pool
pub fn init(widget_pool: &WidgetPool, sim_rd: Arc<Mutex<RenderData>>) {
    // create render data struct
    let mut render_data = RenderData::new(widget_pool, CAMERA_3D_IMAGE);

    // set its face cube to empty
    render_data.gcube.set_face_cube(FaceCube::empty());

    // move the camera to face up the cube
    render_data
        .camera
        .look_at(Point3::new(0.0, 0.0, 9.0), Point3::origin());

    // first rendering to show the cube
    render_data.render();

    // arc and mutex are used to share our render data between thread used by buttons signals
    let rd_arc = Arc::new(Mutex::new(render_data));

    // connect signals
    connect_signals(widget_pool, rd_arc, sim_rd);
}

/// Connect all the signals of **Camera View**
///
/// ## Args
///
/// * `wp` - the gtk widget pool
/// * `rd_arc` - the render data
///
/// ## Note
///
/// It will also add a signal on `SIM_CAMERA_BUTTON`.
fn connect_signals(
    wp: &WidgetPool,
    rd_arc: Arc<Mutex<RenderData>>,
    sim_rd: Arc<Mutex<RenderData>>,
) {
    // rc of the camera
    let camera_rc: Rc<RefCell<Option<rscam::Camera>>> = Rc::new(RefCell::new(None));

    // we keep the following ids to remove the signals when the window is closes
    // the id of the space signal
    let space_id: Rc<RefCell<Option<SignalHandlerId>>> = Rc::new(RefCell::new(None));
    let space_id_clone = Rc::clone(&space_id);

    // the structs that we use to store detection datas
    // color table is used to store the link between tags (from Cube or Gcube) to rgb values
    // rgb data is used to store the rgb value of each facelet
    let color_table = Arc::new(Mutex::new(ColorTable::new()));
    let rgb_data = Rc::new(RefCell::new(RgbData::new()));

    // phase counter
    let phase = Arc::new(Mutex::new(0));
    let phase_clone = phase.clone();

    // ---------- MOUSE SIGNALS ----------

    connect_mouse_signals(
        wp,
        rd_arc.clone(),
        color_table.clone(),
        rgb_data.clone(),
        phase.clone(),
    );

    // ---------- COLOR BUTTONS ----------

    // get the box for buttons
    let button_box: gtk::Box = wp.get(CAMERA_BOTTOM_BOX);

    // get its children
    let children: Vec<gtk::Button> = button_box
        .children()
        .iter()
        .filter_map(|child| child.clone().downcast::<gtk::Button>().ok())
        .collect();

    // for each button, when it is clicked, remove the corresponding color of the color table
    for (index, button) in children.iter().enumerate() {
        let ct_clone = color_table.clone();
        button.connect_clicked(move |_| {
            let mut ct = ct_clone.lock().unwrap();
            if index < ct.get_valid_color().len() {
                ct.remove(index);
            }
        });
    }

    // ---------- MAIN CAMERA BUTTON ----------

    // rc clones
    let rd_clone = rd_arc.clone();
    let rgb_clone = rgb_data.clone();
    let ct_clone = color_table.clone();
    let wp_clone = wp.clone();
    let camera_clone = camera_rc.clone();

    // simulator camera button signal
    // the open view signal is set to the same button in simulator connect_signals function
    wp.get_button(SIM_CAMERA_BUTTON).connect_clicked(move |_| {
        // reset rd datas
        if let Some(mut rd) = rd_clone.try_lock().ok() {
            // reset cube rotation
            rd.gcube.node.set_local_rotation(UnitQuaternion::identity());
            let rotation = UnitQuaternion::from_euler_angles(0.0, PI, 0.0);
            rd.gcube.append_rotation(&rotation);

            // reste datas
            rd.gcube.set_face_cube(FaceCube::empty());
            // render the cube
            rd.render();
        } else {
            error!("Cannot access render data to reset it after cube detection.");
        }

        // reset phase counter
        *phase.lock().unwrap() = 0;

        // hide the progress bar
        let progress_bar: gtk::ProgressBar = wp_clone.get(CAMERA_PROGRESS_BAR);
        progress_bar.set_visible(false);

        // ---------- POPOVER SET UP ----------

        // get the corresponding pop over
        let gtk_popover: gtk::Popover = wp_clone.get(CAMERA_3D_POP_OVER);

        // attach it to the gtk image
        gtk_popover.set_relative_to(Some(&rd_clone.lock().unwrap().gtk_image));

        // ---------- SPACE KEY EVENT ----------

        // arc clone
        // bis are because we are in a second layer of move functions
        let phase_clone_bis = phase_clone.clone();
        let rd_clone_bis = rd_clone.clone();
        let ct_clone_bis = ct_clone.clone();
        let wp_clone_bis = wp_clone.clone();
        let sim_rd_clone_bis = sim_rd.clone();
        let rgb_clone_bis = rgb_clone.clone();
        let camera_clone_bis = camera_clone.clone();
        let si_clone_bis = space_id_clone.clone();

        // create the space clicked connection
        // we do it inside the SIM CAMERA BUTOON signal because we want to remove it after this view is quitted
        let window: gtk::Window = wp_clone.get(MAIN_WINDOW);
        *space_id_clone.borrow_mut() =
            Some(window.connect_key_press_event(move |_widget, event| {
                // space key is pressed
                if event.keyval() == gtk::gdk::keys::constants::space {
                    if let Some(mut phase) = phase_clone_bis.try_lock().ok() {
                        // got to next step if enough color detected
                        if *phase == 0 && ct_clone_bis.lock().unwrap().get_valid_color().len() == 6
                        {
                            // next step
                            *phase = 1;

                            // show progress bar and fill it
                            let progress_bar: gtk::ProgressBar =
                                wp_clone_bis.get(CAMERA_PROGRESS_BAR);
                            progress_bar.set_visible(true);
                            progress_bar.set_fraction(1.0 / 6.0);

                            // set info label
                            let info_label: gtk::Label = wp_clone_bis.get(CAMERA_INFO_LABEL);
                            info_label.set_text("Show your cube, press 'Space' to go next step.");

                            // debug infos
                            info!("Color calibration: done");
                            info!("Starting cube detection.");
                        } else if *phase == 6 {
                            // cube detection is done
                            info!("Cube detection: done");

                            // send detected cube to sim view
                            if let Some(_rd) = rd_clone_bis.try_lock().ok() {
                                if let Some(mut sim_rd) = sim_rd_clone_bis.try_lock().ok() {
                                    if let Some(color_table) = ct_clone_bis.try_lock().ok() {
                                        // send data to the main sim view
                                        let _fcube =
                                            rgb_clone_bis.borrow().to_facelet_cube(&color_table);

                                        Parameters::set_camera_color_table(color_table.clone());

                                        //let fcube = fcube.reverse();

                                        Parameters::set_use_defautl_color_table(false);

                                        //let fcube = rd.gcube.face_cube;

                                        let cp = [
                                            Corner::UFL,
                                            Corner::URF,
                                            Corner::ULB,
                                            Corner::UBR,
                                            Corner::DFR,
                                            Corner::DLF,
                                            Corner::DBL,
                                            Corner::DRB,
                                        ];
                                        let co = [2, 1, 1, 1, 1, 1, 1, 1];
                                        let ep = [
                                            Edge::FR,
                                            Edge::UF,
                                            Edge::BL,
                                            Edge::UL,
                                            Edge::DF,
                                            Edge::DR,
                                            Edge::FL,
                                            Edge::BR,
                                            Edge::UR,
                                            Edge::DL,
                                            Edge::DB,
                                            Edge::UB,
                                        ];
                                        let eo = [0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0];

                                        let c = CubieCube::new(cp, co, ep, eo);

                                        sim_rd.gcube.set_face_cube(c.to_facelet_cube());
                                        sim_rd.render();
                                    }
                                } else {
                                    error!(
                                    "Cannot access simulator render data to transfer detected cube."
                                );
                                }
                            }

                            quit_detection(
                                camera_clone_bis.clone(),
                                si_clone_bis.borrow_mut(),
                                &wp_clone_bis,
                                ct_clone_bis.lock().unwrap(),
                                rgb_clone_bis.borrow_mut(),
                            );
                        } else if *phase > 0 {
                            // try accessung the rendering data
                            if let Some(mut rd) = rd_clone_bis.try_lock().ok() {
                                // update progress bar
                                let progress_bar: gtk::ProgressBar =
                                    wp_clone_bis.get(CAMERA_PROGRESS_BAR);
                                progress_bar.set_fraction(((*phase as f64) + 1.0) / 6.0);

                                // get the orientation of the move
                                let (yaw, pitch, roll) = CUBE_MOVES[*phase - 1];
                                // render it
                                rd.render_cube_rotation(yaw, pitch, roll, ROTATION_DURATION_CAMERA);

                                // increase phase counter
                                *phase += 1;
                            } else {
                                warning!(
                                    "Cannot access rendering data to rotate cube in camera view."
                                );
                            }

                            info!(
                                "\nPhase: {} | Detecting Face: {}",
                                *phase,
                                ColorTag::from(FACE_ORDER[*phase - 1])
                            );
                        }
                    }

                    // do not propagate
                    return Propagation::Stop;
                }
                // propagate
                return Propagation::Proceed;
            }));

        // ---------- MAIN LOOP ----------

        // try accessing the camera
        if let Ok(mut camera) = rscam::Camera::new(&Parameters::camera_path()) {
            // start it
            if let Ok(_) = camera.start(
                &(Config {
                    interval: (1, 30),
                    resolution: (CAMERA_WIDTH, CAMERA_HEIGHT),
                    format: b"RGB3",
                    ..Default::default()
                }),
            ) {
                // pack the camera to close it when this view is closed
                *camera_clone.borrow_mut() = Some(camera);

                // rc clones
                let camera_clone = camera_clone.clone();
                let rd_clone = rd_clone.clone();
                let wp_clone = wp_clone.clone();
                let ct_clone = ct_clone.clone();
                let rgb_clone = rgb_clone.clone();
                let phase_clone = phase_clone.clone();

                // gtk widgets
                let image: gtk::Image = wp_clone.get(CAMERA_CAPTURE_IMAGE);
                let info_label: gtk::Label = wp_clone.get(CAMERA_INFO_LABEL);

                // frame count used to purge periodically the detected facelets
                let mut frame_count = 0_u32;

                info!("Phase: 0, Starting color calibration.");

                // loop
                gtk::glib::idle_add_local(move || {
                    // if there is a camera open
                    if let Some(camera) = camera_clone.borrow_mut().as_mut() {
                        // update frame count
                        frame_count += 1;

                        // capture a frame
                        let frame = camera.capture().unwrap();

                        // create an image from the frame
                        let mut img = imagery::image::Image::new_frome_frame(&frame);

                        // mirror it
                        img.mirror_horizontal();

                        // if wera not already using phase
                        if let Some(phase) = phase_clone.try_lock().ok() {
                            // phase 0 : color calibration
                            if *phase == 0 {
                                color_calibration(
                                    &mut img,
                                    &wp_clone,
                                    ct_clone.lock().unwrap(),
                                    frame_count,
                                    info_label.clone(),
                                );
                            } else {
                                // phase 1 to 6: face detection
                                face_detection(
                                    &mut img,
                                    ct_clone.lock().unwrap(),
                                    rgb_clone.borrow_mut(),
                                    rd_clone.clone(),
                                    *phase,
                                );
                            }
                        } else {
                            warning!("Cannot access phase to perform camera logic.");
                        }

                        // convert into bytes the image and draw it in gtk
                        let glib_bytes = gtk::glib::Bytes::from(img.colored_bytes());
                        let pixbuf = gdk_pixbuf::Pixbuf::from_bytes(
                            &glib_bytes,
                            gdk_pixbuf::Colorspace::Rgb,
                            false,
                            8,
                            img.width() as i32,
                            img.height() as i32,
                            (img.width() as i32) * 3,
                        );
                        image.set_from_pixbuf(Some(&pixbuf));
                    } else {
                        // no camera available so we break the loop
                        // it is either the view has been closed or the camera encounter a problem
                        info!("No camera avalaible, breaking the loop.");

                        // we break the loop
                        return gtk::gdk::glib::ControlFlow::Break;
                    }

                    // continue to loop
                    gtk::gdk::glib::ControlFlow::Continue
                });
            } else {
                error!("Cannot start camera.")
            }
        } else {
            error!("Cannot acces camera at {}", Parameters::camera_path())
        }
    });

    // ---------- BACK BUTTON ----------
    // rc clones
    let wp_clone = wp.clone();
    let space_id_clone = space_id.clone();
    let camera_clone = camera_rc.clone();

    // back button signal
    wp.get_button(CAMERA_BACK_BUTTON).connect_clicked(move |_| {
        quit_detection(
            camera_clone.clone(),
            space_id_clone.borrow_mut(),
            &wp_clone,
            color_table.lock().unwrap(),
            rgb_data.borrow_mut(),
        )
    });
}

/// Quit the **Camera View**, close the camera and reset values.
fn quit_detection(
    camera: Rc<RefCell<Option<Camera>>>,
    mut space_id: RefMut<Option<SignalHandlerId>>,
    wp: &WidgetPool,
    mut color_table: MutexGuard<ColorTable>,
    mut rgb_data: RefMut<RgbData>,
) {
    let mut camera_borrow = camera.borrow_mut();

    // Stop the camera if it exists
    if let Some(ref mut camera) = *camera_borrow {
        if camera.stop().is_err() {
            error!("Unable to stop camera.");
        }
        *camera_borrow = None;
    }

    // remove the space signal connection
    if let Some(id) = space_id.take() {
        let window: gtk::Window = wp.get(MAIN_WINDOW);
        window.disconnect(id);
    }

    // reset color table
    *color_table = ColorTable::new();
    *rgb_data = RgbData::new();

    // open simulator view
    open_simulator(&wp);
}

/// Connect all the mouse signals of the **Camera View**
///
/// ## Args
///
/// * `wp` - the widget pool
/// * `rd_arc` - the render data
/// * `color_table` - the color table from the first phase
/// * `rgb data` - the rgb values of the cube detected
/// * `phase` - the current phase
fn connect_mouse_signals(
    wp: &WidgetPool,
    rd_arc: Arc<Mutex<RenderData>>,
    color_table: Arc<Mutex<ColorTable>>,
    rgb_data: Rc<RefCell<RgbData>>,
    phase: Arc<Mutex<usize>>,
) {
    // ---------------------------------------------
    // --------------- MOUSE SIGNALS ---------------
    // ---------------------------------------------

    // get the color selecting pop over
    let gtk_popover: gtk::Popover = wp.get(CAMERA_3D_POP_OVER);
    // create a custom popover
    let custom_popover = CustomPopover::new(gtk_popover);
    let popover = Rc::new(RefCell::new(custom_popover));

    // if the user has clicked on a facelet or not
    let has_selected_facelet = Rc::new(RefCell::new(false));
    // cublet clicked that we want to change the color
    let prev_facelet_hovered: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));

    // ---------- POP OVER SIGNALS ----------

    // get the button color grid of th epopover
    let button_grid: gtk::Grid = wp.get(CAMERA_POP_OVER_GRID);

    // init each button
    for i in 0..3 {
        for j in 0..2 {
            // get the button
            // we take it as a widget because of to_glib_none
            let widget: gtk::Widget = button_grid
                .child_at(i as i32, j as i32)
                .expect(&format!("Grid button not found at position {i:?}, {j:?}"));

            // create a gdk color
            let gdk_color = GdkRGBA {
                red: 0.5 as f64,
                green: 0.5 as f64,
                blue: 0.5 as f64,
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

            // rc clones
            let popover_clone = popover.clone();
            let prev_clone = prev_facelet_hovered.clone();
            let has_selected_clone = has_selected_facelet.clone();
            let rd_clone = rd_arc.clone();
            let ct_clone = color_table.clone();
            let phase_clone = phase.clone();
            let rgb_clone = rgb_data.clone();

            // connect a signal to the button
            button.connect_clicked(move |_| {
                // if we are already at phase 0 do nothing
                let phase = phase_clone.lock().unwrap();
                if *phase > 0 {
                    // access rendering data
                    if let Some(mut rd) = rd_clone.try_lock().ok() {
                        // get the last overed facelet
                        if let Some(facelet) = *prev_clone.borrow() {
                            // get its node
                            if let Some(_facelet_node) = &mut rd.gcube.gfacelets[facelet] {
                                let mut ct = ct_clone.lock().unwrap();
                                // get the color from the color table
                                let color = *ct.get_valid_color()[j * 3 + i];

                                // set color in rd
                                set_facelet_color(&mut rd, facelet, color);

                                // use transpose to get the corresponding facelet in rgb data
                                let transposed = MATRIX[facelet];

                                // set the new color
                                rgb_clone.borrow_mut().set(transposed, color);

                                // it is a center facelet
                                if facelet % 9 == 4 {
                                    // change tag in the color table
                                    info!("changed tag");
                                    ct.set_tag(color, FACE_ORDER[*phase - 1].into());
                                }

                                let tag = ct.tag_from_rgb(color);
                                rd.gcube.face_cube.set_facelet(transposed, tag);

                                // ?
                                //facelet_node.node.set_color(r, g, b);
                            } else {
                                error!("Facelet {} of the cube is not defined.", facelet);
                            }
                        }
                        // render
                        rd.render();
                    } else {
                        error!("Can not access render data to change cube color");
                    }

                    // hide pop over
                    popover_clone.borrow_mut().popdown();

                    // reset has selected
                    *has_selected_clone.borrow_mut() = false;

                    // erase last hovered facelet
                    *prev_clone.borrow_mut() = None;
                }
            });
        }
    }

    // --------------- GLOBAL BOX CLICK ---------------

    // the 3d event box
    let event_box_3d: gtk::EventBox = wp.get(CAMERA_3D_EVENT_BOX);
    // the global event box (all the window)
    let global_event_box: gtk::EventBox = wp.get(CAMERA_GLOBAL_EVENT_BOX);

    let is_clicked = Rc::new(RefCell::new(false));
    // instant when drag started
    let instant_clicked = Rc::new(RefCell::new(Instant::now()));

    // rc clone
    let is_clicked_clone = is_clicked.clone();
    let instant_clicked_clone = instant_clicked.clone();
    let phase_clone = phase.clone();

    // clicked anywhere on the screen
    global_event_box.connect_button_press_event(move |_, _| {
        if *phase_clone.lock().unwrap() > 0 {
            // get current time
            *instant_clicked_clone.borrow_mut() = Instant::now();

            // set is clicked
            *is_clicked_clone.borrow_mut() = true;
            // propagate
            gtk::glib::Propagation::Proceed
        } else {
            gtk::glib::Propagation::Stop
        }
    });

    // --------------- GLOBAL BOX RELEASE ---------------
    // rc clones
    let rd_clone = rd_arc.clone();
    let is_clicked_clone = is_clicked.clone();
    let instant_clicked_clone = instant_clicked.clone();
    let popover_clone = popover.clone();
    let has_selected_clone = has_selected_facelet.clone();
    let prev_clone = prev_facelet_hovered.clone();

    // release anywhere on the screen
    global_event_box.connect_button_release_event(move |_, _| {
        // reset is clicked
        *is_clicked_clone.borrow_mut() = false;

        // if it was clicked not long ago
        if (*instant_clicked_clone.borrow()).elapsed().as_millis() < DELTA_TIME_CLICK {
            // reset has selected
            *has_selected_clone.borrow_mut() = false;

            // switch popover state
            if popover_clone.borrow().is_poped {
                popover_clone.borrow_mut().popdown();
            }

            // change the color of the previously hovered facelet
            if let Some(rd) = &mut rd_clone.try_lock().ok() {
                let mut prev_facelet = prev_clone.borrow_mut();
                if change_facelet_color(rd, *prev_facelet, -DELTA_COLOR) {
                    *prev_facelet = None;
                }
                // render
                rd.render();
            } else {
                warning!("Cannot access render data when clicked.");
            }
        }
        gtk::glib::Propagation::Proceed
    });

    // --------------- MOVMENT 3D BOX ---------------

    // rc clone
    let has_selected_clone = has_selected_facelet.clone();
    let prev_clone = prev_facelet_hovered.clone();
    let is_clicked_clone = is_clicked.clone();
    let rd_clone = rd_arc.clone();

    // a movment is detected inside the 3d image
    event_box_3d.connect_motion_notify_event(move |_, event| {
        if !*is_clicked_clone.borrow() && !*has_selected_clone.borrow() {
            // check if we can access the render data
            // if not, it is probably just that we are already rendering move and so we do not want
            // to do ray casting
            if let Some(rd) = &mut rd_clone.try_lock().ok() {
                let (x, y) = event.root();

                // if it exists, reset the color of the previous facelet hovered
                let mut prev_facelet = prev_clone.borrow_mut();
                if change_facelet_color(rd, *prev_facelet, -DELTA_COLOR) {
                    *prev_facelet = None;
                }

                // check for raycast
                if let Some(facelet) = rd.raycast(x, y) {
                    // if a facelet is hovered, change its color
                    if change_facelet_color(rd, Some(facelet), DELTA_COLOR) {
                        *prev_facelet = Some(facelet);
                    }
                }

                // render
                rd.render();
            } else {
                warning!("Cannot access render data when movment notified.");
            }
        }
        gtk::glib::Propagation::Proceed
    });

    // --------------- RELEASE 3D BOX ---------------
    // rc clone
    let instant_clicked_clone = instant_clicked.clone();
    let rd_clone = rd_arc.clone();
    let ct_clone = color_table.clone();

    // mouse button is released on the 3d renderer
    event_box_3d.connect_button_release_event(move |_, _| {
        // reset is clicked
        *is_clicked.borrow_mut() = false;

        // get previous facelet hovered
        let mut prev_facelet = prev_facelet_hovered.borrow_mut();

        // if it was clicked not long ago
        if (*instant_clicked.borrow()).elapsed().as_millis() < DELTA_TIME_CLICK {
            let mut has_selected = has_selected_facelet.borrow_mut();
            let mut popover = popover.borrow_mut();
            let ct = ct_clone.lock().unwrap();

            // if popover is hidden
            if !popover.is_poped {
                if let Some(_) = *prev_facelet {
                    // switch popover state
                    popover.popup();

                    // update button color from color table
                    for i in 0..3 {
                        for j in 0..2 {
                            // get button
                            // we take it as a widget because of to_glib_none
                            let widget: gtk::Widget = button_grid
                                .child_at(i as i32, j as i32)
                                .expect(&format!("Grid button not found at position {i:?}, {j:?}"));

                            let (r, g, b) = ct.get_color(j * 3 + i).rgb_normalized();

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
                        }
                    }
                    // a facelet is selected
                    *has_selected = true;
                }
            } else {
                // no facelet is selected
                *has_selected = false;

                // hide popover
                popover.popdown();
                if let Some(rd) = &mut rd_clone.try_lock().ok() {
                    // change the colore of the previously hovered facelet
                    if change_facelet_color(rd, *prev_facelet, -DELTA_COLOR) {
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

/// Set the color of the detection color button (at the bottom of the view).
///
/// ## Args
///
/// * `wp` - the widget pool
/// * `index` - the index of the button in the grid
/// * `color` - the new color to set
fn set_detection_button_color(wp: &WidgetPool, index: usize, color: RgbColor) {
    // get the box for buttons
    let button_box: gtk::Box = wp.get(CAMERA_BOTTOM_BOX);

    // get children of the box
    let children = button_box.children();

    // get the button that we want
    let button = &children[index];

    // convert the rgb color into normalized rgb color
    let (r, g, b) = color.rgb_normalized();

    // create a default gdk color
    let gdk_color = GdkRGBA {
        red: r as f64,
        green: g as f64,
        blue: b as f64,
        alpha: 1.0,
    };

    // get a pointer to our widget
    let button_widget_ptr: *mut GtkWidget = button.to_glib_none().0;

    // use c code to override background color
    unsafe {
        gtk_widget_override_background_color(
            button_widget_ptr,
            gtk_sys::GTK_STATE_FLAG_NORMAL,
            &gdk_color,
        );
    }
}

/// Change the color of a facelet depending on the delta color.
///
/// ## Args
///
/// * `render_data` - the rendering data of the view
/// * `facelet` - the facelet to change the color
/// * `delta_color` - the delta color to apply
fn set_facelet_color(
    render_data: &mut MutexGuard<RenderData>,
    facelet: usize,
    rgb_color: RgbColor,
) {
    // check if the facelet is defined
    if let Some(facelet_node) = render_data
        .gcube
        .gfacelets
        .get_mut(facelet)
        .expect("FaceletNode not found")
    {
        // normalize the color
        let (r, g, b) = rgb_color.rgb_normalized();

        // set the new color
        facelet_node.node.set_color(r, g, b);

        // render
        render_data.render();
    } else {
        error!("Facelet is not defined at index: {}", facelet);
    }
}

/// Color calibration iteration (phase 0 in cube detection). Detect color of the cube and update `color table`.
///
/// ## Args
///
/// * `img` - the image captured from the camera
/// * `wp` - the widget pool
/// * `color_table` - the color table to update
/// * `frame_count` - the frame count to know if data purge is needed
/// * `info_label` - label to display information
fn color_calibration(
    img: &mut Image,
    wp: &WidgetPool,
    mut color_table: MutexGuard<ColorTable>,
    frame_count: u32,
    info_label: gtk::Label,
) {
    // calibration of color data
    img.calibrate_color_data(&mut color_table);

    // purge the color data from parasite colors
    // normally it is very rare that there is something to purge
    if frame_count % Parameters::purge_rate() == 0 {
        color_table.purge(Parameters::purge_threshold());
    }

    // get the valid color from the tabe
    let valid_color = color_table.get_valid_color();
    let valid_count = valid_color.len();

    // calibration phase is finished
    if valid_count == 6 {
        info_label.set_text(&format!("Press 'Space' to start scanning the cube."));
    } else {
        // update label
        info_label.set_text(&format!(
            "Show your cube!\nColor detection: {}/6",
            valid_count
        ));
    }

    // reset all the button color
    for i in valid_count..6 {
        set_detection_button_color(wp, i, DEFAULT_BUTTON_COLOR);
    }

    // set the button detection collor
    for (i, color) in valid_color.iter().enumerate() {
        set_detection_button_color(wp, i, **color);
    }
}

/// Face detection iteration (phase 1 to 7 in cube detection). Detect color of the cube and update `rgb data`.
///
/// ## Args
///
/// * `img` - the image captured from the camera
/// * `color_table` - the color table to update
/// * `rgb_data` - the rgb data storing values detected for the cube
/// * `render_data` - the render data of the view
/// * `phase` - the current phase
fn face_detection(
    img: &mut Image,
    mut color_table: MutexGuard<ColorTable>,
    mut rgb_data: RefMut<RgbData>,
    render_data: Arc<Mutex<RenderData>>,
    phase: usize,
) {
    let phase = phase - 1;

    let rgb_tmp = rgb_data.clone();

    // apply color detection
    img.apply_color_detection(&mut color_table, &mut rgb_data, FACE_ORDER[phase]);

    // try get the rendering datas
    if let Some(mut rd) = render_data.try_lock().ok() {
        // go through the facelets of the current face (depending on the phase)
        for x in 0..3 {
            for y in 0..3 {
                // get the current facelet index
                let facelet = x * 3 + y + 9 * FACE_ORDER[phase];

                // use transposition tab to compute the find facelet on the gcube
                let transposed_facelet = CONV_MATRIX[facelet];

                // get the more consistant estimated color for the current facelet
                if let Some(rgb) = rgb_data.get_max(facelet) {
                    // get its tag from the color table
                    let tag = color_table.tag_from_rgb(rgb);

                    // it is a center facelet
                    if facelet % 9 == 4 {
                        // check the current face rgb value
                        if let Some(_rgb_face) = color_table.rgb_from_tag(FACE_ORDER[phase].into())
                        {
                            // if it has changed since last time
                            //if rgb != rgb_face {
                            // change it to the newest
                            color_table.set_tag(rgb, FACE_ORDER[phase].into());
                            // change tag in the cubec
                            //rd.gcube.face_cube.set_facelet(facelet, tag);
                            // set color in rd
                            //set_facelet_color(&mut rd, transposed_facelet, rgb);
                            //}
                        } else {
                            // tag == none
                            // set tag in the table
                            color_table.set_tag(rgb, FACE_ORDER[phase].into());
                            // set tag in the cube
                            //rd.gcube.face_cube.set_facelet(facelet, tag);
                            // set color in rd
                            //set_facelet_color(&mut rd, transposed_facelet, rgb);

                            color_table.print();
                        }
                    }
                    //else if
                    //rd.gcube.face_cube.facelets()[facelet] != tag // if its tag changed then change the color
                    //{
                    //info!("set facelet");
                    //rd.gcube.face_cube.set_facelet(facelet, tag);
                    //set_facelet_color(&mut rd, transposed_facelet, rgb);
                    //}
                    if let Some(rgb_old) = rgb_tmp.get_max(facelet) {
                        if rgb_old != rgb {
                            rd.gcube.face_cube.set_facelet(facelet, tag);
                            set_facelet_color(&mut rd, transposed_facelet, rgb);
                        }
                    } else {
                        rd.gcube.face_cube.set_facelet(facelet, tag);
                        set_facelet_color(&mut rd, transposed_facelet, rgb);
                    }
                } else {
                    warning!("No rgb value for {}", facelet);
                }
            }
        }
    } else {
        warning!("Cannot access render data while detecting the cube.");
    }
}
