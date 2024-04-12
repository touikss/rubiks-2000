extern crate gtk;
extern crate kiss3d;
extern crate nalgebra as na;

use glib::Propagation;
use gtk::atk::CoordType;
use gtk::{gdk_pixbuf, prelude::*, Builder, Button, CssProvider, Label};

use rscam::Config;
use std::env;
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::{cell::RefCell, process::Command};
use ui::imagery::{self, color::ColorType};
mod cube;
mod graphic_cube;

use cube::cube::Cube;
use graphic_cube::{
    gcube::{Gcube, Move::*, ANIMATION_TIME},
    gtk_binding::update_gtk_image,
};

use kiss3d::{camera::FirstPerson, light::Light, ncollide3d::math::Point, window::Window};

const TIMER_ID: &str = "timer_view";
const MENU_ID: &str = "menu_view";
const SOLVER_ID: &str = "main_view";
const CAMERA_ID: &str = "camera_view";
const SOLUTION_VIEW: &str = "solution_view";
const VIEWS_ID: [&str; 5] = [TIMER_ID, MENU_ID, SOLVER_ID, CAMERA_ID, SOLUTION_VIEW];

const CAMERA_WIDTH: u32 = 640;
const CAMERA_HEIGHT: u32 = 480; //640
const RENDER_RATE: u64 = 500;

const INFO_STRINGS: [&str; 11] = [
    "Modify your capture (click on facelets you want to change)",
    "Turn your cube to the right and align it with the green squares",
    "Modify your capture (click on facelets you want to change)",
    "Turn your cube to the right and align it with the green squares",
    "Modify your capture (click on facelets you want to change)",
    "Turn your cube to the right and align it with the green squares",
    "Modify your capture (click on facelets you want to change)",
    "Turn your cube upwards and align it with the green squares",
    "Modify your capture (click on facelets you want to change)",
    "Turn your cube upwards TWICE and align it with the green squares",
    "Modify your capture (click on facelets you want to change)",
];

fn show_view(builder: &Builder, view: &str) {
    for v in VIEWS_ID {
        let gtk_view: gtk::Viewport = builder.object(v).expect("Couldn't get view stack");
        if v != view {
            gtk_view.set_visible(false);
        } else {
            gtk_view.set_visible(true);
        }
    }
}

fn remove_css_class(button: &Button) {
    // remove other class style
    for c in ColorType::iter() {
        button
            .style_context()
            .remove_class(&(c.to_string() + &String::from("-button")));
    }
}

// Open the camera view
fn open_camera(builder: &Builder) -> Option<Cube> {
    // Show view
    show_view(builder, CAMERA_ID);

    // Open camera
    let mut camera = rscam::Camera::new("/dev/video0").unwrap();
    camera
        .start(&Config {
            interval: (1, 30),
            resolution: (CAMERA_WIDTH, CAMERA_HEIGHT),
            format: b"RGB3",
            ..Default::default()
        })
        .unwrap();

    // Get image
    let image: gtk::Image = builder
        .object("camera_image")
        .expect("no camera image found");

    // Get info label
    let label: gtk::Label = builder
        .object("camera_info_label")
        .expect("no camera image found");
    label.set_label("Show a face to your camera");

    // bool for ok button
    let is_next_pressed = Rc::new(RefCell::new(0));
    let is_next_pressed_clone = Rc::clone(&is_next_pressed);

    // ok callback
    let camera_next_button: gtk::Button = builder
        .object("camera_next_button")
        .expect("no d button found");

    let camera_next_button_ref = camera_next_button.clone();
    let ok_image = &camera_next_button_ref
        .child()
        .unwrap()
        .downcast::<gtk::Box>()
        .unwrap()
        .children()[1];
    ok_image.set_visible(false);
    camera_next_button.connect_clicked(move |_| {
        // increase step count
        *is_next_pressed_clone.borrow_mut() += 1;

        let camera_image = &camera_next_button_ref
            .child()
            .unwrap()
            .downcast::<gtk::Box>()
            .unwrap()
            .children()[0];
        let ok_image = &camera_next_button_ref
            .child()
            .unwrap()
            .downcast::<gtk::Box>()
            .unwrap()
            .children()[1];
        // update image
        if *is_next_pressed_clone.borrow_mut() % 2 == 0 {
            camera_image.set_visible(true);
            ok_image.set_visible(false);
        } else {
            camera_image.set_visible(false);
            ok_image.set_visible(true);
        }

        if *is_next_pressed_clone.borrow_mut() <= 11 {
            // update info label
            label.set_label(INFO_STRINGS[*is_next_pressed_clone.borrow_mut() - 1]);
        }
    });

    // bool for back button
    let is_back_pressed = Rc::new(RefCell::new(false));
    let is_back_pressed_clone = Rc::clone(&is_back_pressed);

    // back callback
    let camera_back_button: gtk::Button = builder
        .object("camera_back_button")
        .expect("no d button found");
    camera_back_button.connect_clicked(move |_| {
        *is_back_pressed_clone.borrow_mut() = true;
    });

    // Display grid
    let display_grid: gtk::Grid = builder
        .object("camera_display_grid")
        .expect("no button grid found");

    // Selection grid
    let selection_grid: gtk::Grid = builder
        .object("camera_selection_grid")
        .expect("no button grid found");
    selection_grid.set_visible(false);

    // the selected button the user wants to modify
    let selected_button: Rc<RefCell<Option<Button>>> = Rc::new(RefCell::new(None));

    // load css provider
    let css_provider = CssProvider::new();
    css_provider
        .load_from_path("src/style.css")
        .expect("Failed to load CSS");

    // the cube constructed
    let cube = Rc::new(RefCell::new(Cube::new_empty(3)));

    // step counter
    let step_counter = Rc::new(RefCell::new(0));

    // init buttons
    for i in 0..3 {
        for j in 0..3 {
            let button = Rc::new(RefCell::new(
                display_grid
                    .child_at(i as i32, j as i32)
                    .expect("nop")
                    .downcast::<Button>()
                    .unwrap(),
            ));

            // init css
            let style_context = button.borrow_mut().style_context();
            style_context.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

            // add callback for display buttons
            let button_clone = button.clone();
            let selection_grid_clone = selection_grid.clone();
            let selected_button_clone = Rc::clone(&selected_button);
            button.borrow_mut().connect_clicked(move |_| {
                // show selection grid
                selection_grid_clone.set_visible(true);
                // change color
                let button = button_clone.borrow_mut();
                button.style_context().add_class("Empty-button");
                // change selected button
                let selected_button_clone = Rc::clone(&selected_button_clone);
                *selected_button_clone.borrow_mut() = Some(button.clone());
            });

            // init selection buttons
            if let Some(color) = ColorType::iter().nth((i * 3 + j) as usize) {
                // Update color tab
                let step_counter_clone = Rc::clone(&step_counter);
                let cube_clone = Rc::clone(&cube);
                cube_clone
                    .borrow_mut()
                    .set_facelet(*step_counter_clone.borrow(), i, j, color);

                let select_button: Button = selection_grid
                    .child_at(i as i32, j as i32)
                    .expect("pas assez de boutton pour select")
                    .downcast::<Button>()
                    .unwrap();

                let selected_button_clone = Rc::clone(&selected_button);
                let selection_grid_clone = selection_grid.clone();
                select_button.connect_clicked(move |_| {
                    // hide selection grid
                    selection_grid_clone.set_visible(false);
                    let selected_button_clone = Rc::clone(&selected_button_clone);
                    remove_css_class(&selected_button_clone.borrow_mut().as_ref().unwrap());
                    selected_button_clone
                        .borrow_mut()
                        .as_ref()
                        .unwrap()
                        .style_context()
                        .add_class(&(color.to_string() + &String::from("-button")));
                });

                let style_context = select_button.style_context();
                style_context.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

                select_button
                    .style_context()
                    .add_class(&(color.to_string() + &String::from("-button")));
            }
        }
    }

    // squares dim
    let length = 20;
    let margin = 70;
    let delta = margin + length;

    let start_x = (CAMERA_WIDTH as f32 / 2. - length as f32 * 3. / 2. - margin as f32) as u32;
    let start_y = (CAMERA_HEIGHT as f32 / 2. - length as f32 * 3. / 2. - margin as f32) as u32;

    // Start loop for capturing and displaying images
    for _ in 0..6 {
        // first loop capturing the colors
        loop {
            // quit
            let is_back_pressed_clone = Rc::clone(&is_back_pressed);
            let is_next_pressed_clone = Rc::clone(&is_next_pressed);
            if *is_back_pressed_clone.borrow_mut() || *is_next_pressed_clone.borrow_mut() > 11 {
                show_view(builder, SOLVER_ID);
                camera.stop().unwrap();
                break;
            }

            // break this loop and go next step
            let is_next_pressed_clone = Rc::clone(&is_next_pressed);
            if *is_next_pressed_clone.borrow_mut() % 2 == 1 {
                break;
            }

            // capture frame
            let frame = camera.capture().unwrap();
            let mut img = imagery::image::Image::new_frome_frame(&frame);

            // draw green squares
            for i in 0..3 {
                for j in 0..3 {
                    img.draw_square(start_x + i * delta, start_y + j * delta, length);
                }
            }

            // compute and analyze with really complicated calculous which color it is
            for i in 0..3 {
                for j in 0..3 {
                    // compute color
                    let color =
                        img.get_square_color(start_x + i * delta, start_y + j * delta, length);

                    let step_counter_clone = Rc::clone(&step_counter);
                    let color_tab_clone = Rc::clone(&cube);
                    color_tab_clone.borrow_mut().set_facelet(
                        *step_counter_clone.borrow(),
                        i,
                        j,
                        color,
                    );

                    // get button
                    let button: Button = display_grid
                        .child_at(i as i32, j as i32)
                        .unwrap()
                        .downcast::<Button>()
                        .unwrap();

                    // remove other class style
                    remove_css_class(&button);

                    // change button color
                    button
                        .style_context()
                        .add_class(&(color.to_string() + &String::from("-button")));
                }
            }

            // convert into bytes the image and draw it in gtk
            let glib_bytes = gtk::glib::Bytes::from(img.bytes());
            let pixbuf = gdk_pixbuf::Pixbuf::from_bytes(
                &glib_bytes,
                gdk_pixbuf::Colorspace::Rgb,
                false,
                8,
                img.width() as i32,
                img.height() as i32,
                img.width() as i32 * 3,
            );
            image.set_from_pixbuf(Some(&pixbuf));

            // Process GTK events
            while gtk::events_pending() {
                gtk::main_iteration();
            }
        }

        // quit
        let is_back_pressed_clone = Rc::clone(&is_back_pressed);
        let is_next_pressed_clone = Rc::clone(&is_next_pressed);
        if *is_back_pressed_clone.borrow_mut() || *is_next_pressed_clone.borrow_mut() > 11 {
            break;
        }

        // second loop allowing user to modify captured colors
        loop {
            // quit
            let is_back_pressed_clone = Rc::clone(&is_back_pressed);
            let is_next_pressed_clone = Rc::clone(&is_next_pressed);
            if *is_back_pressed_clone.borrow_mut() || *is_next_pressed_clone.borrow_mut() > 11 {
                break;
            }

            // break this loop and go next step
            let is_next_pressed_clone = Rc::clone(&is_next_pressed);
            if *is_next_pressed_clone.borrow_mut() % 2 == 0 {
                break;
            }

            // Process GTK events
            while gtk::events_pending() {
                gtk::main_iteration();
            }
        }

        // quit
        let is_back_pressed_clone = Rc::clone(&is_back_pressed);
        let is_next_pressed_clone = Rc::clone(&is_next_pressed);
        if *is_back_pressed_clone.borrow_mut() || *is_next_pressed_clone.borrow_mut() > 11 {
            show_view(builder, SOLVER_ID);
            camera.stop().unwrap();
            break;
        }

        // increment step counter
        let step_counter_clone = Rc::clone(&step_counter);
        *step_counter_clone.borrow_mut() += 1;
    }

    // quit
    let is_next_pressed_clone = Rc::clone(&is_next_pressed);
    if *is_next_pressed_clone.borrow_mut() > 11 {
        let cube_clone = Rc::clone(&cube);
        return Some(*cube_clone.borrow());
    }

    None
}

fn init_gtk_buttons(builder: &Builder) {
    let menu_site_button: gtk::Button = builder
        .object("menu_site_button")
        .expect("No site_button found");
    menu_site_button.connect_clicked(move |_| {
        opensite();
    });

    let window: gtk::Window = builder.object("main_window").expect("no main window found");

    // Menu buttons
    let menu_quit_button: gtk::Button = builder
        .object("menu_quit_button")
        .expect("No menu_quit_button found");
    menu_quit_button.connect_clicked(move |_| {
        window.close();
    });

    let menu_timer_button: gtk::Button = builder
        .object("menu_timer_button")
        .expect("No menu_timer_button found");
    let build_clone = builder.clone();
    menu_timer_button.connect_clicked(move |_| {
        open_timer(&build_clone);
        //show_view(&build_clone, TIMER_ID);
    });

    let menu_solver_button: gtk::Button = builder
        .object("menu_solver_button")
        .expect("No menu_solver_button found");
    let build_clone = builder.clone();
    menu_solver_button.connect_clicked(move |_| {
        show_view(&build_clone, SOLVER_ID);
    });

    // main view buttons
    let main_back_button: gtk::Button =
        builder.object("main_back_button").expect("No button found");
    let build_clone = builder.clone();
    main_back_button.connect_clicked(move |_| {
        show_view(&build_clone, MENU_ID);
    });
}

// open timer view
fn open_timer(builder: &Builder) {
    show_view(builder, TIMER_ID);

    let timer_label: gtk::Label = builder.object("timer_label").expect("no timer label found");

    // ptn c'est sale gtk
    // switch button
    let is_switch_pressed = Rc::new(RefCell::new(false));
    let is_switch_pressed_clone = Rc::clone(&is_switch_pressed);

    let timer_switch_button: gtk::Button = builder
        .object("timer_switch_button")
        .expect("no timer button found");
    timer_switch_button.connect_clicked(move |_| {
        let mut is_switch_pressed_clone = is_switch_pressed_clone.borrow_mut();
        *is_switch_pressed_clone = !*is_switch_pressed_clone;
    });

    // back button
    let is_back_pressed = Rc::new(RefCell::new(false));
    let is_back_pressed_clone = Rc::clone(&is_back_pressed);

    let timer_back_button: gtk::Button = builder
        .object("timer_back_button")
        .expect("no timer back button found");
    let build_clone = builder.clone();
    timer_back_button.connect_clicked(move |_| {
        show_view(&build_clone, MENU_ID);
        *is_back_pressed_clone.borrow_mut() = true;
    });

    let mut is_running = false;
    let mut start_time = Instant::now();

    loop {
        if is_running {
            let elapsed_time = start_time.elapsed();
            let seconds = elapsed_time.as_secs();
            let millis = elapsed_time.subsec_millis();
            let time_text = format!("{:02}:{:03}", seconds, millis);
            timer_label.set_text(&time_text);
        }

        // quit la boucle si back pressed
        let is_back_pressed_clone = Rc::clone(&is_back_pressed);
        if *is_back_pressed_clone.borrow_mut() {
            break;
        }

        let is_switch_pressed_clone = Rc::clone(&is_switch_pressed);
        if *is_switch_pressed_clone.borrow_mut() {
            if let Some(child) = timer_switch_button.child() {
                let label = child.downcast::<Label>().expect("pas de label pr le timer");
                if !is_running {
                    label.set_label("Stop");
                    start_time = Instant::now();
                } else {
                    label.set_label("Start");
                }
            }
            is_running = !is_running;
            *is_switch_pressed_clone.borrow_mut() = false;
        }

        while gtk::events_pending() {
            gtk::main_iteration();
        }
    }
}

fn main_loop(builder: &Builder) {
    // Créer la fenêtre kiss3D.
    let mut window_3d = Window::new_hidden("Rubik's 2000");
    window_3d.set_light(Light::Absolute(Point::new(0., 0., 0.)));

    // Set background color of 3D rendering
    let main_window: gtk::Window = builder.object("main_window").expect("no maon window");
    let style_context = main_window.style_context();
    if let Some(color) = style_context.lookup_color("theme_bg_color") {
        window_3d.set_background_color(
            color.red() as f32,
            color.green() as f32,
            color.blue() as f32,
        );
    } else {
        window_3d.set_background_color(0., 0., 0.);
    }

    // z pr tourner de gauche à droite
    // la cam regarde en direction du cube
    // 5, 5, 7
    let mut camera = FirstPerson::new(
        Point::new(-5. * 0.8, 5. * 0.8, -7. * 0.8),
        Point::new(0., 0., 0.),
    );

    // le cube à link avec la caméra
    let cube = Rc::new(RefCell::new(Cube::new_filled(3)));

    let cube_clone = Rc::clone(&cube);
    let mut gcube = Gcube::new(*cube_clone.borrow(), &mut window_3d);

    let mut image: gtk::Image = builder
        .object("main_3d_image")
        .expect("no menu image found");

    let window: gtk::Window = builder.object("main_window").expect("no main window found");

    let gtk_exited = Rc::new(RefCell::new(false));
    let gtk_exited_clone = gtk_exited.clone();
    window.connect_delete_event(move |_, _| {
        *gtk_exited_clone.borrow_mut() = true; // Indiquer que GTK a quitté
        gtk::glib::Propagation::Stop
    });

    // a cube is provided by camera view
    let is_camera_captured = Rc::new(RefCell::new(false));
    let is_camera_captured_clone = Rc::clone(&is_camera_captured);

    // camera button
    let main_camera_button: gtk::Button = builder
        .object("main_camera_button")
        .expect("No main camera button found");
    let build_clone = builder.clone();
    let cube_clone = Rc::clone(&cube);
    main_camera_button.connect_clicked(move |_| {
        // set new gcube
        if let Some(new_cube) = open_camera(build_clone.as_ref()) {
            *cube_clone.borrow_mut() = new_cube;
            *is_camera_captured_clone.borrow_mut() = true;
        }
    });

    // Bou c pas boooo

    // F is pressed
    let is_f_pressed = Rc::new(RefCell::new(false));
    let is_f_pressed_clone = Rc::clone(&is_f_pressed);

    let main_f_button: gtk::Button = builder.object("main_f_button").expect("no f button found");
    main_f_button.connect_clicked(move |_| {
        *is_f_pressed_clone.borrow_mut() = true;
    });

    // B is pressed
    let is_b_pressed = Rc::new(RefCell::new(false));
    let is_b_pressed_clone = Rc::clone(&is_b_pressed);

    let main_b_button: gtk::Button = builder.object("main_b_button").expect("no b button found");
    main_b_button.connect_clicked(move |_| {
        *is_b_pressed_clone.borrow_mut() = true;
    });

    // L is pressed
    let is_l_pressed = Rc::new(RefCell::new(false));
    let is_l_pressed_clone = Rc::clone(&is_l_pressed);

    let main_l_button: gtk::Button = builder.object("main_l_button").expect("no l button found");
    main_l_button.connect_clicked(move |_| {
        *is_l_pressed_clone.borrow_mut() = true;
    });

    // R is pressed
    let is_r_pressed = Rc::new(RefCell::new(false));
    let is_r_pressed_clone = Rc::clone(&is_r_pressed);

    let main_r_button: gtk::Button = builder.object("main_r_button").expect("no r button found");
    main_r_button.connect_clicked(move |_| {
        *is_r_pressed_clone.borrow_mut() = true;
    });

    // U is pressed
    let is_u_pressed = Rc::new(RefCell::new(false));
    let is_u_pressed_clone = Rc::clone(&is_u_pressed);

    let main_u_button: gtk::Button = builder.object("main_u_button").expect("no u button found");
    main_u_button.connect_clicked(move |_| {
        *is_u_pressed_clone.borrow_mut() = true;
    });

    // D is pressed
    let is_d_pressed = Rc::new(RefCell::new(false));
    let is_d_pressed_clone = Rc::clone(&is_d_pressed);

    let main_d_button: gtk::Button = builder.object("main_d_button").expect("no d button found");
    main_d_button.connect_clicked(move |_| {
        *is_d_pressed_clone.borrow_mut() = true;
    });

    // shuffle is pressed
    let is_back_pressed = Rc::new(RefCell::new(false));
    let is_shuffle_pressed_clone = Rc::clone(&is_back_pressed);
    let main_shuffle_button: gtk::Button = builder
        .object("main_shuffle_button")
        .expect("no shuffle button found");
    main_shuffle_button.connect_clicked(move |_| {
        *is_shuffle_pressed_clone.borrow_mut() = true;
    });

    // solve is pressed
    let is_solve_pressed = Rc::new(RefCell::new(false));
    let is_solve_pressed_clone = Rc::clone(&is_solve_pressed);
    let main_solve_button: gtk::Button = builder
        .object("main_solve_button")
        .expect("no solve button found");
    main_solve_button.connect_clicked(move |_| {
        *is_solve_pressed_clone.borrow_mut() = true;
    });

    // shuffle is pressed
    let is_solve_pressed = Rc::new(RefCell::new(false));
    let is_shuffle_pressed_clone = Rc::clone(&is_solve_pressed);
    let main_shuffle_button: gtk::Button = builder
        .object("main_shuffle_button")
        .expect("no shuffle button found");
    main_shuffle_button.connect_clicked(move |_| {
        *is_shuffle_pressed_clone.borrow_mut() = true;
    });

    // to controll 3D render frame rate
    let mut last_render_time = Instant::now();

    //3D loop
    loop {
        // shuffle
        let is_shuffle_pressed_clone = Rc::clone(&is_solve_pressed);
        if *is_shuffle_pressed_clone.borrow_mut() {
            gcube.shuffle(&mut window_3d, 40, &mut image, &mut camera);
            *is_shuffle_pressed_clone.borrow_mut() = false;
        }

        // solve
        let is_solve_pressed_clone = Rc::clone(&is_solve_pressed);
        if *is_solve_pressed_clone.borrow_mut() {
            solution_loop(&builder, &mut window_3d)
           
           // *is_solve_pressed_clone.borrow_mut() = false;
        }

        // camera captured
        let is_camera_captured_clone = Rc::clone(&is_camera_captured);
        let cube_clone = Rc::clone(&cube);
        if *is_camera_captured.borrow_mut() {
            gcube.set_cube(*cube_clone.borrow());
            *is_camera_captured_clone.borrow_mut() = false;
        }

        // F is pressed
        let is_f_pressed_clone = Rc::clone(&is_f_pressed);
        if *is_f_pressed_clone.borrow_mut() {
            gcube.apply_move(
                &mut window_3d,
                L,
                0,
                &mut image,
                &mut camera,
                ANIMATION_TIME,
            );
            *is_f_pressed_clone.borrow_mut() = false;
        }

        // B is pressed
        let is_b_pressed_clone = Rc::clone(&is_b_pressed);
        if *is_b_pressed_clone.borrow_mut() {
            gcube.apply_move(
                &mut window_3d,
                L,
                2,
                &mut image,
                &mut camera,
                ANIMATION_TIME,
            );
            *is_b_pressed_clone.borrow_mut() = false;
        }

        // U is pressed
        let is_u_pressed_clone = Rc::clone(&is_u_pressed);
        if *is_u_pressed_clone.borrow_mut() {
            gcube.apply_move(
                &mut window_3d,
                H,
                2,
                &mut image,
                &mut camera,
                ANIMATION_TIME,
            );
            *is_u_pressed_clone.borrow_mut() = false;
        }

        // D is pressed
        let is_d_pressed_clone = Rc::clone(&is_d_pressed);
        if *is_d_pressed_clone.borrow_mut() {
            gcube.apply_move(
                &mut window_3d,
                H,
                0,
                &mut image,
                &mut camera,
                ANIMATION_TIME,
            );
            *is_d_pressed_clone.borrow_mut() = false;
        }

        // L is pressed
        let is_l_pressed_clone = Rc::clone(&is_l_pressed);
        if *is_l_pressed_clone.borrow_mut() {
            gcube.apply_move(
                &mut window_3d,
                V,
                2,
                &mut image,
                &mut camera,
                ANIMATION_TIME,
            );
            *is_l_pressed_clone.borrow_mut() = false;
        }

        // R is pressed
        let is_r_pressed_clone = Rc::clone(&is_r_pressed);
        if *is_r_pressed_clone.borrow_mut() {
            gcube.apply_move(
                &mut window_3d,
                V,
                0,
                &mut image,
                &mut camera,
                ANIMATION_TIME,
            );
            *is_r_pressed_clone.borrow_mut() = false;
        }

        while gtk::events_pending() {
            gtk::main_iteration();
        }

        // Render kiss3D view
        // pas trop souvent, sinon les ordis fragiles de timothé et de arthur vont pas le surpporter
        if Instant::now().duration_since(last_render_time) >= Duration::from_millis(RENDER_RATE) {
            last_render_time = Instant::now();
            let gtk_exited_clone = Rc::clone(&gtk_exited);
            if !window_3d.render_with_camera(&mut camera) || *gtk_exited_clone.borrow() {
                break;
            }

            // link between kiss3D and gtk
            update_gtk_image(&mut window_3d, &mut image);
        }
    }
}

fn solution_loop(builder: &Builder, window_3d: &mut Window) {
    

}

fn opensite() {
    let local_site_folder = "../site-web/index.html";
    let _ = Command::new("xdg-open").arg(local_site_folder).output();
}

fn main() {
    gtk::init().expect("Failed to initialize GTK.");

    // force dark theme
    gtk::Settings::default()
        .expect("Failed to get GTK settings")
        .set_gtk_application_prefer_dark_theme(true);

    // Take ui.glade path
    let glade_path = if let Ok(current_dir) = env::current_dir() {
        if let Some(path_str) = current_dir.to_str() {
            path_str.to_string() + "/res/ui.glade"
        } else {
            panic!("Unable to convert path in characters");
        }
    } else {
        panic!("Unable to find current directory");
    };

    let builder = gtk::Builder::from_file(glade_path);
    let gtk_window: gtk::Window = builder.object("main_window").expect("No window found");
    gtk_window.maximize();
    gtk_window.show_all();

    init_gtk_buttons(&builder);

    main_loop(&builder);
}
