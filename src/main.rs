// modules
pub mod cube;
pub mod graphic_cube;
pub mod gui;
pub mod imagery;
pub mod solver;

// std
use std::{
    env,
    sync::{Arc, Mutex, MutexGuard},
};

// local
use cube::{defs::*, enums::ColorTag};
use gui::{menu::open_menu, solution::Solution, utils::MAIN_WINDOW, widget_pool::WidgetPool};
use imagery::{
    color::{ColorTable, RgbColor},
    utils::{BLUE, GRAY, GREEN, ORANGE, RED, WHITE, YELLOW},
};

// other
use lazy_static::lazy_static;

use gtk::prelude::*;

const PURGE_RATE_MAX: u32 = 40;

// The singleton we use to store parameters and settings
pub struct Parameters {
    // SIMULATOR
    /// Wether the camera position will be reset after drag or not
    reset_camera_after_drag: bool,
    /// Wether all the move button will be shown
    advanced_mode: bool,
    /// The number of moves used to shuffle the [`GCube`]
    shuffle_length: u32,

    // CUBE DETECTION
    /// The sensitivity of the detection
    detection_sensitivity: f64,
    /// The purge rate of color table
    purge_rate: u32,
    /// The threshold of color table
    purge_threshold: u32,
    /// The camera device path
    camera_path: String,

    // SOLVER
    /// The timeout of the solver
    solver_timeout: f64,
    /// The max moves that the solver will search
    solver_max_moves: u32,

    // SOLUTION
    /// Wether to show or not preview moves
    /// This is not in parameters only a global internal variable
    show_preview: bool,
    /// The speed of the solution in auto-run mode
    solution_speed: u32,
    /// The animation duration of moves in solution view
    solution_animation_duration: f32,
    ///
    show_infos: bool,

    // GLOBAL
    /// Wether to show warning and info messages or not
    verbose: bool,
    debug: bool,
    default_color_table: ColorTable,
    camera_color_table: ColorTable,
    use_default_color_table: bool,
}

// we use getter and setter to keep the arc mutex locked only in a small scope
impl Parameters {
    pub fn new() -> Self {
        let mut default_ct = ColorTable::new();

        default_ct.add_with_tag(WHITE, ColorTag::F);
        default_ct.add_with_tag(RED, ColorTag::R);
        default_ct.add_with_tag(BLUE, ColorTag::U);
        default_ct.add_with_tag(GREEN, ColorTag::D);
        default_ct.add_with_tag(YELLOW, ColorTag::B);
        default_ct.add_with_tag(ORANGE, ColorTag::L);

        Parameters {
            // SIMULATOR
            reset_camera_after_drag: true,
            advanced_mode: false,
            shuffle_length: 20,

            // CUBE DETECTION
            detection_sensitivity: 50.0,
            purge_rate: 20,
            purge_threshold: 20,
            camera_path: "/dev/video0".to_string(),

            // SOLVER
            solver_timeout: 100.0,
            solver_max_moves: 20,

            // SOLUTION
            show_preview: true,
            solution_speed: 5,
            solution_animation_duration: 0.8,
            show_infos: true,

            // GLOBAL
            verbose: false,
            debug: false,
            default_color_table: default_ct,
            camera_color_table: ColorTable::new(),
            use_default_color_table: true,
        }
    }

    fn instance() -> MutexGuard<'static, Parameters> {
        SINGLETON.lock().unwrap()
    }

    // Getters
    pub fn reset_camera_after_drag() -> bool {
        Self::instance().reset_camera_after_drag
    }

    pub fn advanced_mode() -> bool {
        Self::instance().advanced_mode
    }

    pub fn shuffle_length() -> u32 {
        Self::instance().shuffle_length
    }

    pub fn detection_sensitivity() -> f64 {
        Self::instance().detection_sensitivity
    }

    pub fn purge_rate() -> u32 {
        Self::instance().purge_rate
    }

    pub fn purge_threshold() -> u32 {
        Self::instance().purge_threshold
    }

    pub fn camera_path() -> String {
        Self::instance().camera_path.clone()
    }

    pub fn solver_timeout() -> f64 {
        Self::instance().solver_timeout
    }

    pub fn solver_max_moves() -> u32 {
        Self::instance().solver_max_moves
    }

    pub fn show_preview() -> bool {
        Self::instance().show_preview
    }

    pub fn solution_speed() -> u32 {
        Self::instance().solution_speed
    }

    pub fn solution_animation_duration() -> f32 {
        Self::instance().solution_animation_duration
    }

    pub fn show_infos() -> bool {
        Self::instance().show_infos
    }

    pub fn verbose() -> bool {
        Self::instance().verbose
    }

    pub fn debug() -> bool {
        Self::instance().debug
    }

    pub fn use_default_color_table() -> bool {
        Self::instance().use_default_color_table
    }

    pub fn default_color_from_tag(tag: ColorTag) -> RgbColor {
        if let Some(rgb) = Self::instance().default_color_table.rgb_from_tag(tag) {
            return rgb;
        } else {
            // critical!("Default color table from parameters is not defined.");
            GRAY
        }
    }

    pub fn color_from_tag(tag: ColorTag) -> RgbColor {
        if let Some(rgb) = Self::instance().camera_color_table.rgb_from_tag(tag) {
            return rgb;
        } else {
            // critical!("Default color table from parameters is not defined.");
            GRAY
        }
    }

    // Setters
    pub fn set_reset_camera_after_drag(new_value: bool) {
        let mut instance = Self::instance();
        instance.reset_camera_after_drag = new_value;
    }

    pub fn set_advanced_mode(new_value: bool) {
        let mut instance = Self::instance();
        instance.advanced_mode = new_value;
    }

    pub fn set_shuffle_length(new_value: u32) {
        let mut instance = Self::instance();
        instance.shuffle_length = new_value;
    }

    pub fn set_detection_sensitivity(new_value: f64) {
        let f = new_value / 100.0;

        let mut instance = Self::instance();
        instance.purge_rate = ((PURGE_RATE_MAX as f64) * f) as u32;

        if instance.purge_rate == 0 {
            instance.purge_rate = 1;
        }

        instance.purge_threshold = PURGE_RATE_MAX - (((PURGE_RATE_MAX as f64) * f) as u32);
        instance.detection_sensitivity = new_value;
    }

    pub fn set_purge_rate(new_value: u32) {
        let mut instance = Self::instance();
        instance.purge_rate = new_value;
    }

    pub fn set_purge_threshold(new_value: u32) {
        let mut instance = Self::instance();
        instance.purge_threshold = new_value;
    }

    pub fn set_camera_path(new_value: String) {
        let mut instance = Self::instance();
        instance.camera_path = new_value;
    }

    pub fn set_solver_timeout(new_value: f64) {
        let mut instance = Self::instance();
        instance.solver_timeout = new_value;
    }

    pub fn set_solver_max_moves(new_value: u32) {
        let mut instance = Self::instance();
        instance.solver_max_moves = new_value;
    }

    pub fn set_show_preview(new_value: bool) {
        let mut instance = Self::instance();
        instance.show_preview = new_value;
    }

    pub fn set_solution_speed(new_value: u32) {
        let mut instance = Self::instance();
        instance.solution_speed = new_value;
    }

    pub fn set_solution_animation_time(new_value: f32) {
        let mut instance = Self::instance();
        instance.solution_animation_duration = new_value;
    }

    pub fn set_show_infos(new_value: bool) {
        let mut instance = Self::instance();
        instance.show_infos = new_value;
    }

    pub fn set_camera_color_table(new_value: ColorTable) {
        let mut instance = Self::instance();
        instance.camera_color_table = new_value;
    }

    pub fn set_use_defautl_color_table(new_value: bool) {
        let mut instance = Self::instance();
        instance.use_default_color_table = new_value;
    }

    pub fn set_verbose(new_value: bool) {
        let mut instance = Self::instance();
        instance.verbose = new_value;
    }

    pub fn set_debug(new_value: bool) {
        let mut instance = Self::instance();
        instance.debug = new_value;
    }
}

lazy_static! {
    pub static ref SINGLETON: Arc<Mutex<Parameters>> = Arc::new(Mutex::new(Parameters::new()));
}

pub mod debug {
    #[macro_export]
    /// Print a warning in terminal (only if verbose or debug is set to true)
    macro_rules! warning {
        ($($arg:tt)*) => {
        {
            use termion::{
                color::{self},
                style::{self},
            };

            use crate::Parameters;

            if Parameters::verbose() || Parameters::debug(){
                eprintln!("{}* {}Warning{} at {}:{}: {}{}{}", style::Bold, color::Fg(color::LightYellow), style::Reset, file!(), line!(), style::Bold, format_args!($($arg)*), style::Reset);
            }
        }
        };
    }

    #[macro_export]
    /// Print an error in the terminal.
    /// It will not panic.
    macro_rules! error {
        ($($arg:tt)*) => {
        {
            use termion::{
                color::{self},
                style::{self},
            };
            eprintln!("{}* {}Error{} at {}:{}: {}{}{}", style::Bold, color::Fg(color::Red), style::Reset, file!(), line!(), style::Bold, format_args!($($arg)*), style::Reset);
        }
        };
    }

    #[macro_export]
    /// Print an error in the terminal and exit the program.
    macro_rules! critical {
        ($($arg:tt)*) => {
        {
            use termion::{
                color::{self},
                style::{self},
            };

            use std::process;

            eprintln!("{}* {}Critical Error{} (exit) at {}:{}: {}{}{}", style::Bold, color::Fg(color::Red), style::Reset, file!(), line!(), style::Bold, format_args!($($arg)*), style::Reset);
            process::exit(1);
        }
        };
    }

    #[macro_export]
    /// Print an information in the terminal (only if verbose or debug is set to true)
    macro_rules! info {
        ($($arg:tt)*) => {
        {
            use termion::{
                color::{self},
                style::{self},
            };
            use crate::Parameters;
            if Parameters::verbose() || Parameters::debug() {
                eprintln!("{}* {}Info{} at {}:{}: {}{}{}", style::Bold, color::Fg(color::LightCyan), style::Reset, file!(), line!(), style::Bold, format_args!($($arg)*), style::Reset);
            }
        }
        };
    }

    #[macro_export]
    /// Print an information in the terminal (only if debug is set to true)
    macro_rules! debug {
        ($($arg:tt)*) => {
        {
            use termion::{
                color::{self},
                style::{self},
            };
            use crate::Parameters;
            if Parameters::debug(){
                eprintln!("{}* {}Debug{} at {}:{}: {}{}{}", style::Bold, color::Fg(color::Magenta), style::Reset, file!(), line!(), style::Bold, format_args!($($arg)*), style::Reset);
            }
        }
        };
    }
}

fn main() {
    // kiss3d (GLFW) shows its hidden windows on wayland: force X11/XWayland
    env::remove_var("WAYLAND_DISPLAY");
    env::set_var("GDK_BACKEND", "x11");

    Parameters::set_verbose(true);

    // set it for debug
    // Parameters::set_debug(true)

    Parameters::set_solver_timeout(2.);
    Parameters::set_solver_max_moves(20);

    // init gtk
    if let Ok(_) = gtk::init() {
        info!("Gtk initialization done successfully");
    } else {
        critical!("Gtk initialization failed");
    }

    // force dark theme
    gtk::Settings::default()
        .expect("Failed to get GTK settings")
        .set_gtk_application_prefer_dark_theme(true);

    // Take ui.glade path
    let glade_path = if let Ok(current_dir) = env::current_dir() {
        if let Some(path_str) = current_dir.to_str() {
            path_str.to_string() + "/res/ui.glade"
        } else {
            panic!() //critical!("Unable to convert path into string.")
        }
    } else {
        panic!() //critical!("Unable to find current directory");
    };

    // create gtk builder from glade file
    let builder = gtk::Builder::from_file(glade_path);

    let widgets = WidgetPool::new(builder);

    // get gtk window
    let gtk_window: gtk::Window = widgets.get(MAIN_WINDOW);

    // maximize window
    gtk_window.maximize();
    gtk_window.show_all();

    gtk_window.connect_delete_event(|_, _| {
        gtk::main_quit();
        info!("Exited application.");
        gtk::gdk::glib::Propagation::Proceed
    });

    connect_signals(&widgets);

    open_menu(&widgets);

    gtk_window.show_all();

    info!("Starting main loop");

    gtk::main();
}

fn connect_signals(widget_pool: &WidgetPool) {
    let solution_arc = Arc::new(Mutex::new(Solution::new_empty()));
    gui::solution::init(widget_pool, solution_arc.clone());
    let sim_rd = gui::simulator::init(widget_pool, solution_arc);
    gui::camera::init(widget_pool, sim_rd);
    gui::help::connect_signals(widget_pool);
    gui::timer::connect_signals(widget_pool);
    gui::menu::connect_signals(widget_pool);
    gui::parameters::connect_signals(widget_pool);
}
