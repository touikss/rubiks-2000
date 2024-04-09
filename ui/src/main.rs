extern crate gtk;
use gtk::{false_, prelude::*, Builder};
use gtk::{Application, ApplicationWindow, Box, Button, Label};
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

fn open_main(builder: &Builder) {
    let menu_view: gtk::Viewport = builder
        .object("menu_view")
        .expect("Couldn't get view stack");
    let main_view: gtk::Viewport = builder
        .object("main_view")
        .expect("Couldn't get view stack");
    let timer_view: gtk::Viewport = builder
        .object("rubikronometer_view")
        .expect("Couldn't get view stack");
    menu_view.set_visible(false);
    main_view.set_visible(true);
    timer_view.set_visible(false);
}

fn open_timer(builder: &Builder) {
    let menu_view: gtk::Viewport = builder
        .object("menu_view")
        .expect("Couldn't get view stack");
    let main_view: gtk::Viewport = builder
        .object("main_view")
        .expect("Couldn't get view stack");
    let timer_view: gtk::Viewport = builder
        .object("rubikronometer_view")
        .expect("Couldn't get view stack");
    menu_view.set_visible(false);
    main_view.set_visible(false);
    timer_view.set_visible(true);
}

fn open_menu(builder: &Builder) {
    let menu_view: gtk::Viewport = builder
        .object("menu_view")
        .expect("Couldn't get view stack");
    let main_view: gtk::Viewport = builder
        .object("main_view")
        .expect("Couldn't get view stack");
    let timer_view: gtk::Viewport = builder
        .object("rubikronometer_view")
        .expect("Couldn't get view stack");
    menu_view.set_visible(true);
    main_view.set_visible(false);
    timer_view.set_visible(false);
}

fn main() {
    let app = Application::new(Some("com.rubiks3000.app"), Default::default());

    /*app.connect_startup(|app| {
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
    });*/

    app.connect_activate(|app| {
        // Création de la fenêtre de menu
        let builder = Rc::new(gtk::Builder::from_file(
            "/mnt/c/Users/artbo/rubiks-2000/ui/ui.glade",
        ));

        let window: gtk::Window = builder.object("main_window").expect("Non");

        //Menu buttons
        let quit_button: gtk::Button = builder.object("quit_button").expect("Pas bouton");
        quit_button.connect_clicked(|_| {
            gtk::main_quit();
        });

        let timer_button: gtk::Button = builder.object("timer_button").expect("Pas bouton");

        let build_clone = builder.clone();
        timer_button.connect_clicked(move |_| {
            open_timer(build_clone.as_ref());
        });

        let build_clone = builder.clone();
        let main_button: gtk::Button = builder.object("solver_button").expect("Pas bouton");
        main_button.connect_clicked(move |_| {
            open_main(build_clone.as_ref());
        });

        let build_clone = builder.clone();
        //main buttons
        let menu_button: gtk::Button = builder.object("menu_button").expect("Pas bouton");
        menu_button.connect_clicked(move |_| {
            open_menu(build_clone.as_ref());
        });

        //timer buttons
        let build_clone = builder.clone();
        let menu_button2: gtk::Button = builder.object("menu_button2").expect("Pas bouton");
        menu_button2.connect_clicked(move |_| {
            open_menu(build_clone.as_ref());
            print!("oui\n")
        });

        let start_button: gtk::Button = builder.object("start_button").expect("Pas bouton");
        let label :gtk::Label = builder.object("timer").expect("pas de label");
        let chronometer = Rc::new(RefCell::new(Chronometer::new()));
        let chronometer_clone = Rc::clone(&chronometer);
        let bol = RefCell::new(true);

        let bol_clone = bol.clone();
        let label_clone = label.clone();
        // Utiliser bol_clone à l'intérieur de la fermeture
        start_button.connect_clicked(move |_| {
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




        window.set_application(Some(app));
        window.show_all();
    });

    app.run();
}
