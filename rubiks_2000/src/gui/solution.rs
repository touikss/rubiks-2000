//! This module handle the **Soluti View**

// std
use std::{ cell::RefCell, rc::Rc, sync::{ Arc, Mutex }, time::{ Duration, Instant } };

// local
use crate::{ cube::{ cubie_cube::CubieCube, enums::Move }, error, warning, Parameters, DEFAULT };

use super::{ render_data::{ MoveType, RenderData }, utils::*, widget_pool::WidgetPool };

// other
use gtk::{
    gdk::EventType,
    glib::{ timeout_add_local, ObjectExt, SignalHandlerId, SourceId },
    prelude::{ ButtonExt, GtkWindowExt, LabelExt, RangeExt, WidgetExt },
};

/// The solution found by the solver, send between the **Simulator View** and **Solution View**.
///
/// ## Notes
///
/// It is made to be used as an iterator :
///
/// ```
///
/// let solution = Solution::new();
///
/// next_mv = solution.next()
/// prev_mv = solution.previous()
///
/// ```
#[derive(Clone)]
pub struct Solution {
    /// The moves of the solution
    moves: Vec<Move>,
    /// The current move pointed by the solution
    current: usize,
    /// The cube starting permutation
    cube: CubieCube,
}

impl Solution {
    /// Create a new empty solution.
    ///
    /// ## Notes
    ///
    /// `cube` is set to [DEFAULT]
    pub fn new_empty() -> Self {
        Self {
            moves: vec![],
            current: 0,
            cube: DEFAULT,
        }
    }

    /// Create a new solution.
    ///
    /// ## Args
    ///
    /// * `moves` - the moves of the solution
    /// * `cube` - the starting cube of the solution
    pub fn new(moves: Vec<Move>, cube: CubieCube) -> Self {
        Self {
            moves,
            current: 0,
            cube,
        }
    }

    /// Get the next move of the solution
    pub fn next(&mut self) -> Option<Move> {
        if self.current < self.moves.len() {
            let res = Some(self.moves[self.current]);
            self.current += 1;
            return res;
        }
        None
    }

    /// Get the inverse previous move of the solution
    pub fn previous(&mut self) -> Option<Move> {
        if self.current > 0 {
            self.current -= 1;
            return Some(self.moves[self.current].inverse());
        }
        None
    }

    /// Get the move to previw of the solution.
    ///
    /// ## Notes
    ///
    /// It is the same as next but without incrementing `current`.
    pub fn to_preview(&mut self) -> Option<Move> {
        if self.moves.len() != 0 && self.current < self.moves.len() {
            return Some(self.moves[self.current]);
        }
        None
    }

    /// Go back to start move.
    pub fn reset(&mut self) {
        self.current = 0;
    }
}

/// Open the **Solution View** and set up information pannel.
pub fn open_solution(widget_pool: &WidgetPool) {
    // show the solution view
    open_view(widget_pool, SOLUTION_ID);

    // setup info pannel
    if Parameters::show_infos() {
        let window: gtk::ApplicationWindow = widget_pool.get(SOL_INFOS_WINDOW);
        let main_window: gtk::Window = widget_pool.get(MAIN_WINDOW);

        let (main_w, main_h) = main_window.size();
        let (window_w, window_h) = window.size();

        if window.is_visible() {
            window.hide();
        } else {
            window.set_visible(true);
            window.show_all();
            window.move_(main_w - window_w - 200, main_h / 2 - window_h / 2);
        }
    }
}

/// Init 3d rendering of the **Solution View**
///
/// ## Args
///
/// * `wp` - the widget pool
/// * `solution_arc` - the solution whivh is shared between simulator view and solution view
///
/// ## Notes
///
/// Using arc mutex allow us to init view only once.
pub fn init(wp: &WidgetPool, solution_arc: Arc<Mutex<Solution>>) {
    // create render data struct
    let mut render_data = RenderData::new(wp, SOL_3D_IMAGE);
    // first rendering to show the cube
    render_data.render();

    // arc and mutex are used to share our render data between thread used by buttons signals
    let rd_arc = Arc::new(Mutex::new(render_data));

    // connect all signals
    connect_signals(wp, rd_arc.clone(), solution_arc);
}

/// Connect all signals of **Solution View**
///
/// ## Args
///
/// `wp` - the widget pool
/// `render_data` - the render data of the **Solution View**
/// `solution` - the shared solution found by the solver
fn connect_signals(
    wp: &WidgetPool,
    render_data: Arc<Mutex<RenderData>>,
    solution: Arc<Mutex<Solution>>
) {
    // last time next was pressed
    let last_next = Arc::new(Mutex::new(Instant::now()));
    // id of the idle func
    let idle_id: Rc<RefCell<Option<SourceId>>> = Rc::new(RefCell::new(None));
    // id of the finish idle func
    let finish_id: Arc<Mutex<Option<SourceId>>> = Arc::new(Mutex::new(None));
    // id of the space signal handling
    let space_id: Rc<RefCell<Option<SignalHandlerId>>> = Rc::new(RefCell::new(None));

    // arc an rc clones
    let rd_clone = render_data.clone();
    let last_next_clone = last_next.clone();
    let sol_clone = solution.clone();
    let idle_id_clone = idle_id.clone();
    let finish_id_clone = finish_id.clone();
    let space_id_clone = space_id.clone();

    // widgets
    let sol_label: gtk::Label = wp.get(SOL_LABEL);
    let main_window: gtk::Window = wp.get(MAIN_WINDOW);
    let solution_scale: gtk::Scale = wp.get(SOL_SCALE);

    // connect a signal to the simulator solve button
    // it will be called after the cube is solved
    // connect the space event and init solution view
    wp.get_button(SIM_SOLUTION_BUTTON).connect_event_after(move |_, event| {
        if event.event_type() == EventType::ButtonRelease {
            // if we can access rendering
            if let Some(mut rd) = rd_clone.try_lock().ok() {
                // reset the cube
                rd.reset_cube_color();

                // get the solution
                let mut solution = sol_clone.lock().unwrap();

                // set the rd cube
                rd.gcube.set_face_cube(solution.cube.to_facelet_cube());

                // reset the scale
                solution_scale.set_range(0.0, solution.moves.len() as f64);

                // preview move
                if let Some(mv) = solution.to_preview() {
                    sol_label.set_text(&mv.to_string());
                    solution_scale.set_value(0.0);
                    rd.highlight_cube_slice(mv);
                    rd.render_preview_move(mv.reduce());
                }
                rd.render();
            } else {
                // cannot apply a move and rotate same time for example
                error!("Cannot access rendering datas to set up solution view.");
            }

            // restart last next instant
            if let Some(mut last_next) = last_next_clone.try_lock().ok() {
                *last_next = Instant::now();
            }

            // arc clones
            let rd_clone_bis = rd_clone.clone();
            let sol_clone_bis = sol_clone.clone();
            let last_next_clone_bis = last_next_clone.clone();

            // connect the idle signal which will display preview move
            *idle_id_clone.borrow_mut() = Some(
                timeout_add_local(Duration::from_millis(1500), move || {
                    if let Some(last_next) = last_next_clone_bis.try_lock().ok() {
                        // we pressed next long ago
                        if Instant::now() - *last_next >= Duration::from_millis(2000) {
                            // access render data
                            if let Some(mut rd) = rd_clone_bis.try_lock().ok() {
                                // acces solution
                                if let Some(mut solution) = sol_clone_bis.try_lock().ok() {
                                    // get preview move
                                    if let Some(mv) = solution.to_preview() {
                                        // render it
                                        // we use reduce to display 2-moves as 1-moves
                                        if mv != Move::Default {
                                            rd.render_preview_move(mv.reduce());
                                        }
                                    }
                                }
                            } else {
                                // cannot apply a move and rotate same time for example
                                warning!(
                                    "Cannot access rendering datas to show previous move in solution view."
                                );
                            }
                        }
                    }

                    gtk::glib::ControlFlow::Continue
                })
            );

            // arc clones
            let finish_id_clone_bis = finish_id_clone.clone();
            let rd_clone_bis = rd_clone.clone();
            let sol_clone_bis = sol_clone.clone();
            let last_next_clone_bis = last_next_clone.clone();

            // widgets
            let sol_scale_bis = solution_scale.clone();
            let sol_label_bis = sol_label.clone();

            // connect space signal
            *space_id_clone.borrow_mut() = Some(
                main_window.connect_key_press_event(move |_widget, event| {
                    // space key is pressed
                    if event.keyval() == gtk::gdk::keys::constants::space {
                        // access solution
                        if let Some(mut solution) = sol_clone_bis.try_lock().ok() {
                            // remove finish id if needed
                            if let Some(mut finish_id) = finish_id_clone_bis.try_lock().ok() {
                                if let Some(id) = finish_id.take() {
                                    id.remove();
                                    *finish_id = None;
                                    // show preview next time
                                    Parameters::set_show_preview(true);
                                }
                            }
                            // access rd
                            if let Some(mut rd) = rd_clone_bis.try_lock().ok() {
                                {
                                    // show next move
                                    if let Some(mv) = solution.next() {
                                        sol_scale_bis.set_value(solution.current as f64);
                                        rd.render_move(mv, MoveType::Solution);
                                        rd.reset_cube_color();
                                    }
                                    // show previex move
                                    if let Some(mv) = solution.to_preview() {
                                        sol_label_bis.set_text(&mv.to_string());
                                        rd.highlight_cube_slice(mv);
                                        rd.render_preview_move(mv.reduce());
                                    } else {
                                        sol_label_bis.set_text("Solved!");
                                    }
                                }

                                // reset last next instant
                                if let Some(mut last_next) = last_next_clone_bis.try_lock().ok() {
                                    *last_next = Instant::now();
                                }
                            } else {
                                // cannot apply a move and rotate same time for example
                                warning!(
                                    "Cannot access rendering datas to show next move in solution view."
                                );
                            }
                        }
                    }
                    gtk::glib::Propagation::Stop
                })
            );
        }
    });

    // arc clones
    let rd_clone = render_data.clone();
    let sol_clone = solution.clone();
    let finish_id_clone = finish_id.clone();

    // widgets
    let sol_label: gtk::Label = wp.get(SOL_LABEL);
    let sol_scale: gtk::Scale = wp.get(SOL_SCALE);

    // previous button signal
    // show previous move in solution
    wp.get_button(SOL_PREV_BUTTON).connect_clicked(move |_| {
        // access finish id
        if let Some(mut finish_id) = finish_id_clone.try_lock().ok() {
            // remove it
            if let Some(id) = finish_id.take() {
                id.remove();
                *finish_id = None;
                Parameters::set_show_preview(true);
            } else if let Some(mut solution) = sol_clone.try_lock().ok() {
                // access rendering
                if let Some(mut rd) = rd_clone.try_lock().ok() {
                    // display previous move
                    if let Some(mv) = solution.previous() {
                        sol_scale.set_value(solution.current as f64);
                        rd.reset_cube_color();
                        rd.highlight_cube_slice(mv);
                        rd.render_move(mv, MoveType::Solution);
                    }

                    // set label
                    if let Some(mv) = solution.to_preview() {
                        sol_label.set_text(&mv.to_string());
                    }
                } else {
                    // cannot apply a move and rotate same time for example
                    warning!(
                        "Cannot access rendering datas to show previous move in solution view, maybe multiple signals are received same time."
                    );
                }
            }
        }
    });

    // arc clones
    let sol_clone = solution.clone();
    let last_next_clone = last_next.clone();
    let rd_clone = render_data.clone();
    let finish_id_clone = finish_id.clone();

    // widgets
    let sol_label: gtk::Label = wp.get(SOL_LABEL);
    let sol_scale: gtk::Scale = wp.get(SOL_SCALE);

    // next button signal
    // show next move in solution
    wp.get_button(SOL_NEXT_BUTTON).connect_clicked(move |_| {
        // acces finish id
        if let Some(mut fid) = finish_id_clone.try_lock().ok() {
            // remove id
            if let Some(id) = fid.take() {
                id.remove();
                *fid = None;
                Parameters::set_show_preview(true);
            } else if
                // access solution
                let Some(mut solution) = sol_clone.try_lock().ok()
            {
                // access rendering
                if let Some(mut rd) = rd_clone.try_lock().ok() {
                    {
                        // show next move
                        if let Some(mv) = solution.next() {
                            sol_scale.set_value(solution.current as f64);
                            rd.render_move(mv, MoveType::Solution);
                            rd.reset_cube_color();
                        }
                        // show preview move
                        if let Some(mv) = solution.to_preview() {
                            sol_label.set_text(&mv.to_string());
                            rd.highlight_cube_slice(mv);
                            rd.render_preview_move(mv.reduce());
                        } else {
                            sol_label.set_text("Solved!");
                        }
                    }

                    // update last next instant
                    if let Some(mut last_next) = last_next_clone.try_lock().ok() {
                        *last_next = Instant::now();
                    }
                } else {
                    // cannot apply a move and rotate same time for example
                    warning!("Cannot access rendering datas to show next move in solution view.");
                }
            }
        }
    });

    // arc clones
    let sol_clone = solution.clone();
    let rd_clone = render_data.clone();
    let finish_id_clone = finish_id.clone();

    // widgets
    let sol_label: gtk::Label = wp.get(SOL_LABEL);
    let sol_scale: gtk::Scale = wp.get(SOL_SCALE);

    // restart button signal
    wp.get_button(SOL_RESTART_BUTTON).connect_clicked(move |_| {
        // access finish id
        if let Some(mut finsih_id) = finish_id_clone.try_lock().ok() {
            // remove the id
            if let Some(id) = finsih_id.take() {
                id.remove();
                *finsih_id = None;
                Parameters::set_show_preview(true);
            } else if
                // access solution
                let Some(mut solution) = sol_clone.try_lock().ok()
            {
                // access rendering
                if let Some(mut rd) = rd_clone.try_lock().ok() {
                    // reset the cube
                    rd.gcube.set_face_cube(solution.cube.to_facelet_cube());
                    solution.reset();

                    // show preview move
                    if let Some(mv) = solution.to_preview() {
                        rd.highlight_cube_slice(mv);
                        sol_label.set_text(&mv.to_string());
                    }
                    sol_scale.set_value(solution.current as f64);
                } else {
                    error!("Unable to access render data to restart solution.");
                }
            }
        }
    });

    // arc clones
    let sol_clone = solution.clone();
    let rd_clone = render_data.clone();
    let finish_id_clone = finish_id.clone();

    // widgets
    let sol_scale: gtk::Scale = wp.get(SOL_SCALE);
    let wp_clone = wp.clone();

    // finsih button signal
    // add timeout function to display moves till the end
    wp.get_button(SOL_FINISH_BUTTON).connect_clicked(move |_| {
        // access solution
        if let Some(mut solution) = sol_clone.try_lock().ok() {
            // do not show preview moves
            Parameters::set_show_preview(false);

            // arc clones
            let rd_clone_bis = rd_clone.clone();
            let sol_clone_bis = sol_clone.clone();
            let finish_id_clone_bis = finish_id_clone.clone();

            // widgets
            let sol_label: gtk::Label = wp_clone.get(SOL_LABEL);
            let scale_clone = sol_scale.clone();

            // access finish id
            if let Some(mut finish_id) = finish_id_clone.try_lock().ok() {
                // access rendering
                if let Some(mut rd) = rd_clone.try_lock().ok() {
                    // display next move
                    if let Some(mv) = solution.next() {
                        rd.render_move(mv, MoveType::Solution);
                        sol_scale.set_value(solution.current as f64);
                    }
                    // display preview move
                    if let Some(mv) = solution.to_preview() {
                        sol_label.set_text(&mv.to_string());
                        rd.reset_cube_color();
                        rd.highlight_cube_slice(mv);
                    }
                }

                // add idle function with timeout
                *finish_id = Some(
                    timeout_add_local(
                        Duration::from_millis(Parameters::solution_speed() as u64),
                        move || {
                            // access rendering
                            if let Some(mut rd) = rd_clone_bis.try_lock().ok() {
                                // reset cube color after move
                                rd.reset_cube_color();

                                // access solution
                                if let Some(mut solution) = sol_clone_bis.try_lock().ok() {
                                    // display next move
                                    if let Some(mv) = solution.next() {
                                        rd.highlight_cube_slice(mv);
                                        rd.render_move(mv, MoveType::Solution);
                                        scale_clone.set_value(solution.current as f64);
                                    } else {
                                        // stop the loop
                                        // we reached the end
                                        if let Some(mut fid) = finish_id_clone_bis.try_lock().ok() {
                                            *fid = None;
                                        }
                                        return gtk::glib::ControlFlow::Break;
                                    }
                                    // display preview move
                                    if let Some(mv) = solution.to_preview() {
                                        sol_label.set_text(&mv.to_string());
                                        rd.reset_cube_color();
                                        rd.highlight_cube_slice(mv);
                                    }
                                }
                            } else {
                                // cannot apply a move and rotate same time for example
                                warning!(
                                    "Cannot access rendering datas to show newt move in solution view finsih loop."
                                );
                            }

                            gtk::glib::ControlFlow::Continue
                        }
                    )
                );
            }
        }
    });

    // widget
    let wp_clone = wp.clone();

    // arc clones
    let idle_id_clone = idle_id.clone();
    let finish_id_clone = finish_id.clone();
    let space_id_clone = space_id.clone();

    // back button signal
    wp.get_button(SOL_BACK_BUTTON).connect_clicked(move |_| {
        // remove all the id
        if let Some(id) = idle_id_clone.borrow_mut().take() {
            id.remove();
        }
        if let Some(id) = finish_id_clone.lock().unwrap().take() {
            id.remove();
        }
        if let Some(id) = space_id_clone.borrow_mut().take() {
            let window: gtk::Window = wp_clone.get(MAIN_WINDOW);
            window.disconnect(id);
        }

        // reset all the labels
        let thread_label: gtk::Label = wp_clone.get(SOL_THREAD_LABEL);
        let sol_label: gtk::Label = wp_clone.get(SOL_SOL_LABEL);
        let length_label: gtk::Label = wp_clone.get(SOL_LENGTH_LABEL);
        let time_label: gtk::Label = wp_clone.get(SOL_ELLAPSED_LABEL);
        let n_sol_label: gtk::Label = wp_clone.get(SOL_N_SOL_LABEL);
        let move_label: gtk::Label = wp_clone.get(SOL_LABEL);

        thread_label.set_text("0");
        sol_label.set_text("_");
        length_label.set_text("_");
        time_label.set_text("_");
        n_sol_label.set_text("_");
        move_label.set_text("Moves");

        // rset show preview
        Parameters::set_show_preview(true);

        // open simulator view
        super::simulator::open_simulator(&wp_clone);
    });
}
