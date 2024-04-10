extern crate gtk;
extern crate kiss3d;
extern crate nalgebra as na;

use gtk::{false_, gdk_pixbuf, prelude::*, Builder, Image};
use gtk::{Application, ApplicationWindow, Box, Button, Label};
use rscam::Config;
use ui::imagery;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::{env, thread};
mod cube;
mod graphic_cube;
use std::sync::{Arc, Mutex};

use cube::cube::Cube;
use graphic_cube::{
    gcube::{Gcube, Move::*},
    gtk_binding::update_gtk_image,
};

use gtk::prelude::*;
use kiss3d::{
    camera::{Camera, FirstPerson},
    light::Light,
    ncollide3d::math::Point,
    window::Window,
};

struct Chronometer {
    start_time: Option<Instant>,
    total_paused_time: Duration,
    last_pause_time: Option<Instant>,
}

const TIMER_ID: &str = "timer_view";
const MENU_ID: &str = "menu_view";
const SOLVER_ID: &str = "main_view";
const CAMERA_ID: &str = "camera_view";
const VIEWS_ID: [&str; 4] = [TIMER_ID, MENU_ID, SOLVER_ID, CAMERA_ID];

const CAMERA_WIDTH: u32 = 640;
const CAMERA_HEIGHT: u32 = 480;

impl Chronometer {
    fn new() -> Chronometer {
        Chronometer {
            start_time: None,
            total_paused_time: Duration::new(0, 0),
            last_pause_time: None,
        }
    }

    fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    fn pause(&mut self) {
        if let Some(start) = self.start_time {
            self.last_pause_time = Some(Instant::now());
            self.total_paused_time += self.last_pause_time.unwrap() - start;
        }
    }

    fn resume(&mut self) {
        if let Some(pause_time) = self.last_pause_time {
            self.total_paused_time -= pause_time.elapsed();
        }
        self.start_time = Some(Instant::now());
        self.last_pause_time = None;
    }

    fn reset(&mut self) {
        self.start_time = None;
        self.total_paused_time = Duration::new(0, 0);
        self.last_pause_time = None;
    }

    fn elapsed(&self) -> f64 {
        match self.start_time {
            Some(start) => {
                let elapsed = start.elapsed() - self.total_paused_time;
                elapsed.as_secs_f64()
            }
            None => 0.0,
        }
    }
}

fn rubikronometer() {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Rubik'ronometer");
    window.set_default_size(300, 200);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 2);
    window.add(&vbox);

    let label = gtk::Label::new(None);
    vbox.pack_start(&label, true, true, 0);

    let chronometer = Rc::new(RefCell::new(Chronometer::new()));

    let start_button = gtk::Button::with_label("Start");
    start_button.set_widget_name("start_button"); //css

    let chronometer_clone = Rc::clone(&chronometer);
    start_button.connect_clicked(move |_| {
        chronometer_clone.borrow_mut().start();
    });
    vbox.pack_start(&start_button, false, false, 0);

    let stop_button = gtk::Button::with_label("Stop");
    stop_button.set_widget_name("stop_button"); //css
    let label_clone = label.clone();
    let chronometer_clone = Rc::clone(&chronometer);
    stop_button.connect_clicked(move |_| {
        let elapsed_time = chronometer_clone.borrow().elapsed();
        println!("Temps écoulé: {:.2} secondes", elapsed_time);
        label_clone.set_text(&format!(
            "Rubik's effectué en : {:.2} secondes",
            elapsed_time
        ));
    });
    vbox.pack_start(&stop_button, false, false, 0);

    window.show_all();
}

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

// Open the menu view
fn open_camera(builder: &Builder) {
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

    let image: gtk::Image = builder.object("camera_image").expect("no camera image found");

    let is_back_pressed = Rc::new(RefCell::new(false));
    let is_back_pressed_clone = Rc::clone(&is_back_pressed);

    let camera_back_button: gtk::Button = builder.object("camera_back_button").expect("no d button found");
    camera_back_button.connect_clicked(move |_| {
        *is_back_pressed_clone.borrow_mut() = true;
    });

    // Start loop for capturing and displaying images
    loop {

        let is_back_pressed_clone = Rc::clone(&is_back_pressed);
        if *is_back_pressed_clone.borrow_mut() {
            show_view(builder, SOLVER_ID);
            break;
        }

        let frame = camera.capture().unwrap();

        let mut img = imagery::image::Image::new_frome_frame(&frame);

        let length = 50;
        let margin = 20;
        let delta = margin + length;

        let start_x = (CAMERA_WIDTH as f32 / 2. - length as f32 * 3. / 2. - margin as f32) as u32;
        let start_y = (CAMERA_HEIGHT as f32 / 2. - length as f32 * 3. / 2. - margin as f32) as u32;

        img.apply_gray_filter();
        //img.apply_blur(1);
        //img.reduce();
        img.apply_mass_detection();

        for i in 0..3 {
            for j in 0..3 {
                img.draw_square(start_x + i * delta, start_y + j * delta, length);
            }
        }
        
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


}

fn init_gtk_buttons(builder: &Builder) {
    // Menu buttons
    let menu_quit_button: gtk::Button = builder
        .object("menu_quit_button")
        .expect("No menu_quit_button found");
    menu_quit_button.connect_clicked(|_| {
        gtk::main_quit();
    });

    let menu_timer_button: gtk::Button = builder
        .object("menu_timer_button")
        .expect("No menu_timer_button found");
    let build_clone = builder.clone();
    menu_timer_button.connect_clicked(move |_| {
        show_view(&build_clone, TIMER_ID);
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

    let main_camera_button: gtk::Button = builder.object("main_camera_button").expect("No main camera button found");
    let build_clone = builder.clone();
    main_camera_button.connect_clicked(move |_| {
        open_camera(build_clone.as_ref());
    });

    // timer view buttons
    let build_clone = builder.clone();
    let timer_back_button: gtk::Button = builder
        .object("timer_back_button")
        .expect("No button found");
    timer_back_button.connect_clicked(move |_| {
        show_view(&build_clone, MENU_ID);
    });

    let timer_start_button: gtk::Button = builder
        .object("timer_start_button")
        .expect("No button found");
    let timer_label: gtk::Label = builder.object("timer_label").expect("pas de label");
    let chronometer = Rc::new(RefCell::new(Chronometer::new()));
    let chronometer_clone = Rc::clone(&chronometer);
    let bol = RefCell::new(true);

    let bol_clone = bol.clone();
    let label_clone = timer_label.clone();
    // Utiliser bol_clone à l'intérieur de la fermeture
    timer_start_button.connect_clicked(move |_| {
        if *bol_clone.borrow() {
            chronometer_clone.borrow_mut().start();
            *bol_clone.borrow_mut() = false;
        } else {
            let elapsed_time = chronometer_clone.borrow().elapsed();
            println!("Temps écoulé: {:.2} secondes", elapsed_time);
            label_clone.set_text(&format!(" {:.2} s", elapsed_time));
            *bol_clone.borrow_mut() = true;
        }
    });

    // Camera buttons
    let camera_back_button: gtk::Button = builder.object("camera_back_button").expect("No back camera button found");
    let build_clone = builder.clone();
    camera_back_button.connect_clicked(move |_| {
        show_view(&build_clone, SOLVER_ID);
    }); 
}

fn loop_3d(builder: &Builder) {
    // Créer la fenêtre kiss3D.
    let mut window_3d = Window::new_hidden("Rubik's 2000");
    window_3d.set_light(Light::StickToCamera);

    // z pr tourner de gauche à droite
    // la cam regarde en direction du cube 
    let mut camera = FirstPerson::new(Point::new(-10., 10., -14.), Point::new(0., 0., 0.));

    let cube = Cube::new_filled(3);
    let mut gcube = Gcube::new(cube, &mut window_3d);

    let mut image: gtk::Image = builder
        .object("main_3d_image")
        .expect("no menu image found");

    // Bou c pas boooo

    let is_f_pressed = Rc::new(RefCell::new(false));
    let is_f_pressed_clone = Rc::clone(&is_f_pressed);

    let main_f_button: gtk::Button = builder.object("main_f_button").expect("no f button found");
    main_f_button.connect_clicked(move |_| {
        *is_f_pressed_clone.borrow_mut() = true;
    });

    let is_b_pressed = Rc::new(RefCell::new(false));
    let is_b_pressed_clone = Rc::clone(&is_b_pressed);

    let main_b_button: gtk::Button = builder.object("main_b_button").expect("no b button found");
    main_b_button.connect_clicked(move |_| {
        *is_b_pressed_clone.borrow_mut() = true;
    });

    let is_l_pressed = Rc::new(RefCell::new(false));
    let is_l_pressed_clone = Rc::clone(&is_l_pressed);

    let main_l_button: gtk::Button = builder.object("main_l_button").expect("no l button found");
    main_l_button.connect_clicked(move |_| {
        *is_l_pressed_clone.borrow_mut() = true;
    });

    let is_r_pressed = Rc::new(RefCell::new(false));
    let is_r_pressed_clone = Rc::clone(&is_r_pressed);

    let main_r_button: gtk::Button = builder.object("main_r_button").expect("no r button found");
    main_r_button.connect_clicked(move |_| {
        *is_r_pressed_clone.borrow_mut() = true;
    });

    let is_u_pressed = Rc::new(RefCell::new(false));
    let is_u_pressed_clone = Rc::clone(&is_u_pressed);

    let main_u_button: gtk::Button = builder.object("main_u_button").expect("no u button found");
    main_u_button.connect_clicked(move |_| {
        *is_u_pressed_clone.borrow_mut() = true;
    });

    let is_d_pressed = Rc::new(RefCell::new(false));
    let is_d_pressed_clone = Rc::clone(&is_d_pressed);

    let main_d_button: gtk::Button = builder.object("main_d_button").expect("no d button found");
    main_d_button.connect_clicked(move |_| {
        *is_d_pressed_clone.borrow_mut() = true;
    });

    loop {
        let is_f_pressed_clone = Rc::clone(&is_f_pressed);
        if *is_f_pressed_clone.borrow_mut() {
            println!("test");
            gcube.apply_move(&mut window_3d, L, 0, &mut image, &mut camera);
            *is_f_pressed_clone.borrow_mut() = false;
        }

        let is_b_pressed_clone = Rc::clone(&is_b_pressed);
        if *is_b_pressed_clone.borrow_mut() {
            println!("test");
            gcube.apply_move(&mut window_3d, L, 2, &mut image, &mut camera);
            *is_b_pressed_clone.borrow_mut() = false;
        }

        let is_u_pressed_clone = Rc::clone(&is_u_pressed);
        if *is_u_pressed_clone.borrow_mut() {
            println!("test");
            gcube.apply_move(&mut window_3d, H, 2, &mut image, &mut camera);
            *is_u_pressed_clone.borrow_mut() = false;
        }

        let is_d_pressed_clone = Rc::clone(&is_d_pressed);
        if *is_d_pressed_clone.borrow_mut() {
            println!("test");
            gcube.apply_move(&mut window_3d, H, 0, &mut image, &mut camera);
            *is_d_pressed_clone.borrow_mut() = false;
        }

        let is_l_pressed_clone = Rc::clone(&is_l_pressed);
        if *is_l_pressed_clone.borrow_mut() {
            println!("test");
            gcube.apply_move(&mut window_3d, V, 2, &mut image, &mut camera);
            *is_l_pressed_clone.borrow_mut() = false;
        }

        let is_r_pressed_clone = Rc::clone(&is_r_pressed);
        if *is_r_pressed_clone.borrow_mut() {
            println!("test");
            gcube.apply_move(&mut window_3d, V, 0, &mut image, &mut camera);
            *is_r_pressed_clone.borrow_mut() = false;
        }

        update_gtk_image(&mut window_3d, &mut image);

        // Rendre la fenêtre kiss3D
        if !window_3d.render_with_camera(&mut camera) {
            break;
        }
    }
}

fn main() {

    gtk::init().expect("Failed to initialize GTK.");

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
    gtk_window.show_all();

    init_gtk_buttons(&builder);
    loop_3d(&builder);
}
