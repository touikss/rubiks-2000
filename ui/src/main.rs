extern crate gtk;
use gtk::{
    gdk, gio, Application, ApplicationWindow, Box, Button, CssProvider, Label, StyleContext,
};
use gtk::{prelude::*, STYLE_PROVIDER_PRIORITY_APPLICATION};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

struct Chronometer {
    start_time: Option<Instant>,
    total_paused_time: Duration,
    last_pause_time: Option<Instant>,
}

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

fn open_empty_window() {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Enter your cube");
    window.set_default_size(300, 200);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 2);
    window.add(&vbox);

    let camera_button = gtk::Button::with_label("Open Camera🎥");
    camera_button.set_widget_name("start_button"); //css
    vbox.pack_start(&camera_button, false, false, 0);

    window.show_all();
}

fn main() {
    gtk::init().expect("Failed to initialize GTK.");

    let app = Application::new(Some("com.rubiks3000.app"), Default::default());

    app.connect_startup(|app| {
        // The CSS "magic" happens here.
        let provider = gtk::CssProvider::new();
        // Load the CSS file
        let style = include_bytes!("style.css");
        provider.load_from_data(style).expect("Failed to load CSS");
        gtk::StyleContext::add_provider_for_screen(
            &gdk::Screen::default().expect("Error initializing gtk css provider."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });

    app.connect_activate(|app| {
        // Création de la fenêtre de menu
        let menu_window = ApplicationWindow::new(app);
        menu_window.set_title("Menu");
        menu_window.set_default_size(600, 400);

        let menu_box = Box::new(gtk::Orientation::Vertical, 0);
        menu_window.add(&menu_box);

        let open_main_button = Button::with_label("Rubikronometer");
        open_main_button.set_widget_name("menu_button"); //css
        open_main_button.connect_clicked(|_| {
            rubikronometer();
        });
        menu_box.pack_start(&open_main_button, true, true, 5);

        let open_empty_button = Button::with_label("Solver");
        open_empty_button.set_widget_name("menu_button"); //css
        open_empty_button.connect_clicked(|_| {
            open_empty_window();
        });
        menu_box.pack_start(&open_empty_button, true, true, 5);

        let button_exit = Button::with_label("Exit");
        button_exit.set_widget_name("button_exit"); //css
        button_exit.connect_clicked(|_| {
            gtk::main_quit();
        });
        menu_box.pack_start(&button_exit, false, false, 5);

        menu_window.show_all();
    });

    app.run();
}
