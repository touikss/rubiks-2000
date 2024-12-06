//! This module handle the solver multi threading.

// std
use std::{ cmp::min, sync::{ Arc, Mutex }, thread, time::{ Duration, Instant } };

use gtk::prelude::LabelExt;

// local
use crate::{
    cube::{
        coord_cube::CoordCube,
        cubie_cube::CubieCube,
        defs::*,
        enums::Move,
        face_cube::FaceCube,
        utils::{ get_corners_ud_edges_depth3, get_flipslice_twist_depth3 },
    },
    debug,
    error,
    gui::{
        utils::{
            SOL_ELLAPSED_LABEL,
            SOL_LENGTH_LABEL,
            SOL_N_SOL_LABEL,
            SOL_SOL_LABEL,
            SOL_THREAD_LABEL,
        },
        widget_pool::WidgetPool,
    },
    info,
    Parameters,
    DEFAULT,
    N_MOVE,
    U_EDGES_PLUS_D_EDGES_TO_UD_EDGES,
};

struct SolverThread {
    cb_cube: CubieCube,
    co_cube: Option<CoordCube>,
    rot: i32,
    inv: i32,
    sofar_phase1: Option<Vec<Move>>,
    sofar_phase2: Option<Vec<Move>>,
    phase2_done: bool,
    ret_length: i32,
    timeout: f64,
    start_time: Instant,
    cornersave: i32,
    solutions: Arc<Mutex<Vec<Vec<Move>>>>,
    terminated: Arc<Mutex<bool>>,
    shortest_length: Arc<Mutex<i32>>,
}

impl SolverThread {
    fn new(
        cb_cube: CubieCube,
        rot: i32,
        inv: i32,
        ret_length: i32,
        timeout: f64,
        start_time: Instant,
        solutions: Arc<Mutex<Vec<Vec<Move>>>>,
        terminated: Arc<Mutex<bool>>,
        shortest_length: Arc<Mutex<i32>>
    ) -> SolverThread {
        SolverThread {
            cb_cube,
            co_cube: None,
            rot,
            inv,
            sofar_phase1: None,
            sofar_phase2: None,
            phase2_done: false,
            ret_length,
            timeout,
            start_time,
            cornersave: 0,
            solutions,
            terminated,
            shortest_length,
        }
    }

    fn search_phase2(
        &mut self,
        corners: u32,
        ud_edges: u32,
        slice_sorted: u32,
        dist: i8,
        togo_phase2: i32
    ) {
        debug!("\n------ SEARCH PHASE 2 ------");
        debug!(
            "Corners: {}, Ud edges: {}, Slice sorted: {}, Dist: {}, To go phase 2: {}",
            corners,
            ud_edges,
            slice_sorted,
            dist,
            togo_phase2
        );

        if *self.terminated.lock().unwrap() || self.phase2_done {
            debug!("Return phase 2\n");
            return;
        }

        if togo_phase2 == 0 && slice_sorted == 0 {
            debug!("FIRST");

            let mut man = self.sofar_phase1.as_ref().unwrap().clone();

            man.extend(self.sofar_phase2.as_ref().unwrap());

            info!("Man: {:?}", man);

            let mut solutions = self.solutions.lock().unwrap();

            if solutions.is_empty() || solutions.last().unwrap().len() > man.len() {
                info!("updated");

                if self.inv == 1 {
                    man.reverse();
                    man.iter_mut().for_each(|m| {
                        *m = match m {
                            Move::U1 => Move::U3,
                            Move::U2 => Move::U2,
                            Move::U3 => Move::U1,
                            Move::R1 => Move::R3,
                            Move::R2 => Move::R2,
                            Move::R3 => Move::R1,
                            Move::F1 => Move::F3,
                            Move::F2 => Move::F2,
                            Move::F3 => Move::F1,
                            Move::D1 => Move::D3,
                            Move::D2 => Move::D2,
                            Move::D3 => Move::D1,
                            Move::L1 => Move::L3,
                            Move::L2 => Move::L2,
                            Move::L3 => Move::L1,
                            Move::B1 => Move::B3,
                            Move::B2 => Move::B2,
                            Move::B3 => Move::B1,
                            _ => *m,
                        };
                    });
                }
                man.iter_mut().for_each(|m| {
                    *m = match m {
                        Move::Default => Move::Default,
                        _ =>
                            (
                                CONJ_MOVE
                                    [N_MOVE * 16 * (self.rot as usize) + (*m as usize)] as usize
                            ).into(),
                    };
                });
                solutions.push(man.clone());
                *self.shortest_length.lock().unwrap() = man.len() as i32;
            }

            if *self.shortest_length.lock().unwrap() <= self.ret_length {
                *self.terminated.lock().unwrap() = true;
            }
            self.phase2_done = true;
        } else {
            debug!("SECOND");

            for m in Move::iter() {
                match m {
                    | Move::R1
                    | Move::R3
                    | Move::F1
                    | Move::F3
                    | Move::L1
                    | Move::L3
                    | Move::B1
                    | Move::B3 => {
                        // debug!("Continue 1");
                        continue;
                    }
                    _ => {}
                }

                let l = self.sofar_phase2.as_ref().unwrap().len();
                if l > 0 {
                    let diff =
                        (self.sofar_phase2.as_ref().unwrap()[l - 1] as usize) / 3 -
                        (*m as usize) / 3;
                    // debug!("Diff: {}", diff);
                    if diff == 0 || diff == 3 {
                        // debug!("Continue 2");
                        continue;
                    }
                } else {
                    let l = self.sofar_phase1.as_ref().unwrap().len();
                    if l > 0 {
                        let diff =
                            (self.sofar_phase1.as_ref().unwrap()[l - 1] as usize) / 3 -
                            (*m as usize) / 3;
                        if diff == 0 || diff == 3 {
                            //debug!("Continue 3");
                            continue;
                        }
                    }
                }

                //debug!("Always Here!");

                // debug!("I: {}", 18 * (ud_edges as usize) + (*m as usize));

                let corners_new = CORNERS_MOVE[18 * (corners as usize) + (*m as usize)];
                let ud_edges_new = UD_EDGES_MOVE[18 * (ud_edges as usize) + (*m as usize)];
                let slice_sorted_new =
                    SLICE_SORTED_MOVE[18 * (slice_sorted as usize) + (*m as usize)];

                // debug!(
                //     "Corner new: {}, UD edges new: {}, Slice sorted new: {}",
                //     corners_new,
                //     ud_edges_new,
                //     slice_sorted_new
                // );

                let classidx = CORNER_CLASSIDX[corners_new as usize];
                let sym = CORNER_SYM[corners_new as usize];
                let dist_new_mod3 = get_corners_ud_edges_depth3(
                    40320 * (classidx as usize) +
                        (
                            UD_EDGES_CONJ
                                [
                                    (((ud_edges_new as usize) << 4) + (sym as usize)) as usize
                                ] as usize
                        )
                );

                // added

                let dist_new = DISTANCE[3 * (dist as usize) + (dist_new_mod3 as usize)];
                // debug!(
                //     "Classidx: {}, Sym: {}, Dist new mod 3: {}, Dist new: {}",
                //     classidx,
                //     sym,
                //     dist_new_mod3,
                //     dist_new
                // );

                if
                    (
                        dist_new.max(
                            CORNSLICE_DEPTH.lock().unwrap()
                                [24 * (corners_new as usize) + (slice_sorted_new as usize)]
                        ) as i32
                    ) >= togo_phase2
                {
                    continue;
                }

                self.sofar_phase2.as_mut().unwrap().push(*m);
                self.search_phase2(
                    corners_new as u32,
                    ud_edges_new as u32,
                    slice_sorted_new as u32,
                    dist_new,
                    togo_phase2 - 1
                );
                self.sofar_phase2.as_mut().unwrap().pop();
            }
        }
    }
    fn search(&mut self, flip: i32, twist: i32, slice_sorted: i32, dist: i32, togo_phase1: i32) {
        debug!("-------SEARCH--------");
        debug!("Dist: {}", dist);
        //info!("Terminated: {}", *self.terminated.lock().unwrap());
        debug!("Flip: {}, Twist: {}", flip, twist);
        debug!("Slice sorted: {}", slice_sorted);
        debug!("Togo phase 1: {}", togo_phase1);

        if *self.terminated.lock().unwrap() {
            return;
        }

        if togo_phase1 == 0 {
            if
                Instant::now() > self.start_time + Duration::from_secs_f64(self.timeout) &&
                !self.solutions.lock().unwrap().is_empty()
            {
                info!("Timed out");
                *self.terminated.lock().unwrap() = true;
            }

            // debug!("Terminated: {}", *self.terminated.lock().unwrap());

            let m;
            if let Some(last_move) = self.sofar_phase1.as_ref().and_then(|moves| moves.last()) {
                m = *last_move;
            } else {
                m = Move::U1; // value is irrelevant here, no phase 1 moves
            }

            debug!("M: {}", m as usize);

            let mut corners;
            if m == Move::R3 || m == Move::F3 || m == Move::L3 || m == Move::B3 {
                corners = CORNERS_MOVE[(18 * self.cornersave + (m as i32) - 1) as usize];
            } else {
                corners = self.co_cube.as_ref().unwrap().corners as u16;

                for m in self.sofar_phase1.as_ref().unwrap() {
                    corners = CORNERS_MOVE[18 * (corners as usize) + (*m as usize)];
                }
                self.cornersave = corners as i32;

                // corners = self.co_cube
                //     .as_ref()
                //     .map_or(0, |cube| cube.corners)
                //     //.into_iter()
                //     .fold(self.cornersave, |acc, m| corners_move[(18 * acc + m) as usize]);
                // self.cornersave = corners as i32;
            }

            debug!("Corner: {}", corners);

            let togo2_limit = min(
                *self.shortest_length.lock().unwrap() -
                    (self.sofar_phase1.as_ref().unwrap().len() as i32),
                11
            );

            debug!("To go 2 limit: {}", togo2_limit);

            debug!("Index: {}", (24 * (corners as usize) + (slice_sorted as usize)) as usize);

            // added
            //let cornslice = cornslice_depth.lock().unwrap();
            if
                (
                    CORNSLICE_DEPTH.lock().unwrap()
                        [(24 * (corners as usize) + (slice_sorted as usize)) as usize] as i32
                ) >= togo2_limit
            {
                debug!("Return\n");

                return;
            }

            let mut u_edges = self.co_cube.as_ref().map_or(0, |cube| cube.u_edges);
            let mut d_edges = self.co_cube.as_ref().map_or(0, |cube| cube.d_edges);

            if let Some(ref moves) = self.sofar_phase1 {
                for m in moves.iter() {
                    //ud_edges = ud_edges_move[(18 * (ud_edges as i32) + (*m as i32)) as usize];
                    u_edges = U_EDGES_MOVE[18 * u_edges + (*m as usize)] as usize;
                    d_edges = D_EDGES_MOVE[18 * d_edges + (*m as usize)] as usize;
                }
            }

            debug!("U/D edges: {}, {}", u_edges, d_edges);
            let ud_edges =
                U_EDGES_PLUS_D_EDGES_TO_UD_EDGES[(24 * u_edges + (d_edges % 24)) as usize];

            debug!("UD edges: {}", ud_edges);

            let dist2 = self.co_cube
                .as_ref()
                .map_or(0, |_cube| CoordCube::get_depth_phase2(corners as u32, ud_edges as u32));

            debug!("Dist 2: {}", dist2);

            for togo2 in dist2 as i32..togo2_limit {
                self.sofar_phase2 = Some(vec![]);
                self.phase2_done = false;
                self.search_phase2(
                    corners as u32,
                    ud_edges as u32,
                    slice_sorted as u32,
                    dist2,
                    togo2
                );
                if self.phase2_done {
                    debug!("PHASE 2 ALREADY DONE");
                    break;
                }
            }
        } else {
            for m in Move::iter() {
                if
                    dist == 0 &&
                    togo_phase1 < 5 &&
                    (*m == Move::U1 ||
                        *m == Move::U2 ||
                        *m == Move::U3 ||
                        *m == Move::R2 ||
                        *m == Move::F2 ||
                        *m == Move::D1 ||
                        *m == Move::D2 ||
                        *m == Move::D3 ||
                        *m == Move::L2 ||
                        *m == Move::B2)
                {
                    debug!("Continue 1");
                    continue;
                }

                if let Some(last_move) = self.sofar_phase1.as_ref().and_then(|moves| moves.last()) {
                    let diff = (*last_move as i32) / 3 - (*m as i32) / 3;
                    if diff == 0 || diff == 3 {
                        debug!("Continue 2");
                        continue;
                    }
                }

                let flip_new = FLIP_MOVE[(18 * flip + (*m as i32)) as usize];
                let twist_new = TWIST_MOVE[(18 * twist + (*m as i32)) as usize];
                let slice_sorted_new =
                    SLICE_SORTED_MOVE[(18 * slice_sorted + (*m as i32)) as usize];

                debug!("Flip new: {}", flip_new);
                debug!("Twist new: {}", twist_new);
                debug!("Slice sorted new: {}", slice_sorted_new);

                let flipslice = 2048 * ((slice_sorted_new as u32) / 24) + (flip_new as u32); // N_FLIP * (slice_sorted // N_PERM_4) + flip
                let classidx = FLIPSLICE_CLASSIDX[flipslice as usize];
                let sym = FLIPSLICE_SYM[flipslice as usize];

                debug!("Flip slice: {}", flipslice);
                debug!("Classidx: {}", classidx);
                debug!("Sym: {}", sym);

                let dist_new_mod3 = get_flipslice_twist_depth3(
                    (2187 * (classidx as usize) +
                        (
                            TWIST_CONJ[((twist_new << 4) as usize) + (sym as usize)] as usize
                        )) as usize
                );
                debug!("Dist new mod3: {}", dist_new_mod3);

                // added
                let dist_new = DISTANCE[(3 * dist + (dist_new_mod3 as i32)) as usize];

                debug!("Dist new: {}", dist_new);

                if (dist_new as i32) >= togo_phase1 {
                    debug!("Continue 3");
                    continue;
                }

                self.sofar_phase1.as_mut().unwrap().push(*m);
                self.search(
                    flip_new as i32,
                    twist_new as i32,
                    slice_sorted_new as i32,
                    dist_new as i32,
                    togo_phase1 - 1
                );
                self.sofar_phase1.as_mut().unwrap().pop();
            }
        }
    }

    pub fn run(&mut self) {
        debug!("thread run");

        let mut cubie_cube: CubieCube;

        match self.rot {
            0 => {
                debug!("no rotation");
                cubie_cube = CubieCube::new(
                    self.cb_cube.cp.clone(),
                    self.cb_cube.co.clone(),
                    self.cb_cube.ep.clone(),
                    self.cb_cube.eo.clone()
                );
            }
            1 => {
                debug!("conjugation by 120° rotation");
                cubie_cube = CubieCube::new(
                    SYM_CUBE[32].cp.clone(),
                    SYM_CUBE[32].co.clone(),
                    SYM_CUBE[32].ep.clone(),
                    SYM_CUBE[32].eo.clone()
                );
                cubie_cube.multiply(self.cb_cube);
                cubie_cube.multiply(SYM_CUBE[16]);
            }
            2 => {
                debug!("conjugation by 240° rotation");
                cubie_cube = CubieCube::new(
                    SYM_CUBE[16].cp.clone(),
                    SYM_CUBE[16].co.clone(),
                    SYM_CUBE[16].ep.clone(),
                    SYM_CUBE[16].eo.clone()
                );
                cubie_cube.multiply(self.cb_cube);
                cubie_cube.multiply(SYM_CUBE[32]);
            }
            _ => {
                return;
            }
        }

        debug!("End match");

        if self.inv == 1 {
            debug!("inv");
            let mut tmp = DEFAULT;
            cubie_cube.inv_cubie_cube(&mut tmp);
            cubie_cube = tmp;
        }
        self.co_cube = Some(CoordCube::new(Some(&cubie_cube))); // the rotated/inverted cube in coordinate representation

        debug!("end coord");

        if let Some(ref co_cube) = self.co_cube.clone() {
            debug!("GET DEPTH PHASE 1");
            let dist = co_cube.get_depth_phase1();

            debug!("dist: {}", dist);

            for togo1 in dist..20 {
                debug!("SEARCH {}", togo1);

                // iterative deepening, solution has at least dist moves
                self.sofar_phase1 = Some(Vec::new());
                self.search(
                    co_cube.flip as i32,
                    co_cube.twist as i32,
                    co_cube.slice_sorted as i32,
                    dist as i32,
                    togo1 as i32
                );
            }
        }
    }
}

pub fn solve(fc: FaceCube) -> Vec<Move> {
    let cubie_cube = fc.to_cubie_cube();

    // the cube is not valid
    if cubie_cube.verify() != CUBE_OK {
        //
        error!("The cube provided to the solver is not valid.");
        return vec![];
    }

    // thread vec
    let mut threads: Vec<thread::JoinHandle<()>> = vec![];
    // starting time for timeour
    let start_time = Instant::now();

    // shortest length already found
    let shortest_length = Arc::new(Mutex::new(999));
    // the solutions found
    let solutions = Arc::new(Mutex::new(vec![]));
    let terminated = Arc::new(Mutex::new(false));

    let symmetries = cubie_cube.symmetries();

    let tr = if symmetries.iter().any(|&x| [16, 20, 24, 28].contains(&x)) {
        vec![0, 3]
    } else {
        (0..6).collect()
    };

    let tr: Vec<_> = if symmetries.iter().any(|&x| (48..96).contains(&x)) {
        tr.into_iter()
            .filter(|&x| x < 3)
            .collect()
    } else {
        tr
    };

    info!("Creating threads: {:?}", tr);

    //let tr = 1..2;

    for i in tr {
        let mut solver_thread = SolverThread::new(
            cubie_cube,
            i % 3,
            i / 3,
            Parameters::solver_max_moves() as i32,
            Parameters::solver_timeout(),
            start_time,
            Arc::clone(&solutions),
            Arc::clone(&terminated),
            Arc::clone(&shortest_length)
        );

        let handle = thread::spawn(move || {
            solver_thread.run();
        });

        threads.push(handle);

        // info!("Thread {} created: ", i);

        break;
    }

    let mut i = 0;
    for handle in threads {
        let id = handle.thread().id();

        if let Err(_) = handle.join() {
            error!("Thread {:?} panicked.", id);
        } else {
            info!("Thread {} finished.", i);
        }
        i += 1;
    }

    let mut s = String::new();
    let solutions = solutions.lock().unwrap();
    if !solutions.is_empty() {
        for m in solutions.last().unwrap() {
            // the last solution is the shortest
            s.push_str(&format!("{} ", *m));
        }
    }

    format!("{}({}f)", s, s.len() / 3);

    solutions.last().unwrap().to_vec()
}

pub fn solve_spec(fc: FaceCube, wp: &WidgetPool) -> Vec<Move> {
    let cubie_cube = fc.to_cubie_cube();

    // the cube is not valid
    if cubie_cube.verify() != CUBE_OK {
        //
        error!("The cube provided to the solver is not valid.");
        return vec![Move::Default];
    }

    // thread vec
    let mut threads: Vec<thread::JoinHandle<()>> = vec![];
    // starting time for timeour
    let start_time = Instant::now();

    // shortest length already found
    let shortest_length = Arc::new(Mutex::new(999));
    // the solutions found
    let solutions = Arc::new(Mutex::new(vec![]));
    let terminated = Arc::new(Mutex::new(false));

    let symmetries = cubie_cube.symmetries();

    let tr = if symmetries.iter().any(|&x| [16, 20, 24, 28].contains(&x)) {
        vec![0, 3]
    } else {
        (0..6).collect()
    };

    let tr: Vec<_> = if symmetries.iter().any(|&x| (48..96).contains(&x)) {
        tr.into_iter()
            .filter(|&x| x < 3)
            .collect()
    } else {
        tr
    };

    info!("Creating threads: {:?}", tr);

    //let tr = 1..2;ù

    let thread_label: gtk::Label = wp.get(SOL_THREAD_LABEL);
    let l = tr.len();

    for i in tr {
        let mut solver_thread = SolverThread::new(
            cubie_cube,
            i % 3,
            i / 3,
            Parameters::solver_max_moves() as i32,
            Parameters::solver_timeout(),
            start_time,
            Arc::clone(&solutions),
            Arc::clone(&terminated),
            Arc::clone(&shortest_length)
        );

        let handle = thread::spawn(move || {
            solver_thread.run();
        });

        threads.push(handle);

        // info!("Thread {} created: ", i);

        thread_label.set_text(&format!("{}", l));

        break;
    }

    let time_label: gtk::Label = wp.get(SOL_ELLAPSED_LABEL);

   // let mut count = threads.len();

    while gtk::events_pending() {
        gtk::main_iteration();
    }

    //let mut i = 0;
    for handle in threads {
        //let id = handle.thread().id();

        while !handle.is_finished() {
            let ellapsed = start_time.elapsed();
            time_label.set_text(&format!("{:.2}", ellapsed.as_secs_f32()));
            while gtk::events_pending() {
                gtk::main_iteration();
            }
        }

        //count -= 1;
        thread_label.set_text(&format!("{}", 0));
        //info!("Thread {} finished.", i);

        // if let Err(_) = handle.join() {
        //     error!("Thread {:?} panicked.", id);
        // } else {
        //     count -= 1;
        //     thread_label.set_text(&format!("{}", count));
        //     info!("Thread {} finished.", i);
        //     if count == 0 {
        //         ended = true;
        //     }
        // }
    }

    let mut s = String::new();
    let solutions = solutions.lock().unwrap();
    if !solutions.is_empty() {
        for m in solutions.last().unwrap() {
            // the last solution is the shortest
            s.push_str(&format!("{} ", *m));
        }
    }

    format!("{}({}f)", s, s.len() / 3);

    let res = solutions.last().unwrap().to_vec();

    let length_label: gtk::Label = wp.get(SOL_LENGTH_LABEL);
    let sol_label: gtk::Label = wp.get(SOL_SOL_LABEL);
    let n_sol_label: gtk::Label = wp.get(SOL_N_SOL_LABEL);

    n_sol_label.set_text(&format!("{}", solutions.len()));
    length_label.set_text(&format!("{}", res.len()));

    let rs = res
        .iter()
        .map(|mv| mv.to_string())
        .collect::<Vec<String>>()
        .join("");
    let mut f = String::new();
    for (i, c) in rs.char_indices() {
        if i % 2 == 0 && i % 12 == 0 {
            f.push('\n');
        } else if i % 2 == 0 && i != 0 {
            f.push(' ');
        }
        f.push(c);
    }
    sol_label.set_text(&format!("{}", f));
    res
}
