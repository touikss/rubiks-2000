//! This module handle the definition of [`CoordCube`].

use crate::{ cube::utils::get_corners_ud_edges_depth3, debug };
use super::{ cubie_cube::CubieCube, defs::*, enums::Move, utils::get_flipslice_twist_depth3 };

#[derive(Clone)]
/// A cube represented at a coordinate level.
/// All the fields are "encoded" values.
///
/// ## Fields:
///
/// * `twist`: The twist of the corners (0 to 2186).
/// * `flip`: The flip of the edges (0 to 2047).
/// * `slice_sorted`: The sorted slice edges (0 to 494).
/// * `u_edges`: The state of the upper edges (0 to 2767).
/// * `d_edges`: The state of the down edges (0 to 2767).
/// * `corners`: The state of the corners (0 to 40319).
/// * `ud_edges`: The state of the up and down edges (32-bit integer).
/// * `flipslice_classidx`: The class index for the flip/slice combination (16-bit unsigned integer).
/// * `flipslice_sym`: The symmetry of the flip/slice combination (8-bit unsigned integer).
/// * `flipslice_rep`: The representative of the flip/slice combination (32-bit unsigned integer).
/// * `corner_classidx`: The class index for the corners (16-bit unsigned integer).
/// * `corner_sym`: The symmetry of the corners (8-bit unsigned integer).
/// * `corner_rep`: The representative of the corners (16-bit unsigned integer).
pub struct CoordCube {
    pub twist: usize,
    pub flip: usize,
    pub slice_sorted: usize,
    pub u_edges: usize,
    pub d_edges: usize,
    pub corners: usize,
    pub ud_edges: i32,
    pub flipslice_classidx: u16,
    pub flipslice_sym: u8,
    pub flipslice_rep: u32,
    pub corner_classidx: u16,
    pub corner_sym: u8,
    pub corner_rep: u16,
}

impl CoordCube {
    /// Creates a new `CoordCube` instance.
    /// It takes an optional reference to a `CubieCube` and constructs a `CoordCube` based on its data.
    ///
    /// # Args
    ///
    /// * `cc`: An optional reference to a `CubieCube`. If `Some`, the function constructs the `CoordCube`
    ///         based on its data. If `None`, initializes the `CoordCube` with default values.
    ///
    /// # Returns
    ///
    /// A new `CoordCube` instance.
    pub fn new(cc: Option<&CubieCube>) -> Self {
        if let Some(cc) = cc {
            debug!("new coord with cc");

            let twist = cc.get_twist();
            let flip = cc.get_flip();
            let slice_sorted = cc.get_slice_sorted();

            debug!("Twist Flip: {}, {}", twist, flip);
            debug!("Slice sorted: {}", slice_sorted);

            let u_edges = cc.get_u_edges();
            let d_edges = cc.get_d_edges();
            let corners = cc.get_corners();
            let ud_edges: i32 = if slice_sorted < N_PERM_4 { cc.get_ud_edges() as i32 } else { -1 };

            debug!("u edges: {}", u_edges);
            debug!("d edges: {}", d_edges);
            debug!("corners: {}", corners);
            debug!("ud edges: {}", ud_edges);

            let flipslice_classidx = FLIPSLICE_CLASSIDX[N_FLIP * (slice_sorted / N_PERM_4) + flip];
            let flipslice_sym = FLIPSLICE_SYM[N_FLIP * (slice_sorted / N_PERM_4) + flip];
            let flipslice_rep = FLIPSLICE_REP[flipslice_classidx as usize];

            debug!("flipslice_classidx: {}", flipslice_classidx);
            debug!("flipslice_rep: {}", flipslice_rep);
            debug!("flipslice_sym: {}", flipslice_sym);

            let corner_classidx = CORNER_CLASSIDX[corners];
            let corner_sym = CORNER_SYM[corners];
            let corner_rep = CORNER_REP[corner_classidx as usize];

            debug!("corner_classidx: {}", corner_classidx);
            debug!("corner_sym: {}", corner_sym);
            debug!("corner_rep: {}", corner_rep);

            Self {
                twist,
                flip,
                slice_sorted,
                u_edges,
                d_edges,
                corners,
                ud_edges,
                flipslice_classidx,
                flipslice_sym,
                flipslice_rep,
                corner_classidx,
                corner_sym,
                corner_rep,
            }
        } else {
            debug!("new");
            Self {
                twist: SOLVED,
                flip: SOLVED,
                slice_sorted: SOLVED,
                u_edges: 1656,
                d_edges: SOLVED,
                corners: SOLVED,
                ud_edges: -1,
                flipslice_classidx: 0,
                flipslice_sym: 0,
                flipslice_rep: 0,
                corner_classidx: 0,
                corner_sym: 0,
                corner_rep: 0,
            }
        }
    }

    // fn phase1_move(&mut self, m: usize) {
    //     self.twist = twist_move[N_MOVE * self.twist + m] as usize;
    //     self.flip = flip_move[N_MOVE * self.flip + m] as usize;
    //     self.slice_sorted = slice_sorted_move[N_MOVE * self.slice_sorted + m] as usize;
    //     // Optionnel :
    //     self.u_edges = u_edges_move[N_MOVE * self.u_edges + m] as usize; // u_edges and d_edges récupèrent facilement ud_edges
    //     self.d_edges = d_edges_move[N_MOVE * self.d_edges + m] as usize; // si la phase 1 est terminée et que la phase 2 commence
    //     self.corners = corners_move[N_MOVE * self.corners + m] as usize; // Est nécessaire uniquement en phase 2
    //
    //     self.flipslice_classidx = flipslice_classidx[
    //         N_FLIP * (self.slice_sorted / N_PERM_4) + self.flip
    //     ];
    //     self.flipslice_sym = flipslice_sym[N_FLIP * (self.slice_sorted / N_PERM_4) + self.flip];
    //     self.flipslice_rep = flipslice_rep[self.flipslice_classidx as usize] as u32;
    //
    //     self.corner_classidx = corner_classidx[self.corners];
    //     self.corner_sym = corner_sym[self.corners];
    //     self.corner_rep = corner_rep[self.corner_classidx as usize] as u16;
    // }
    //
    // fn phase2_move(&mut self, m: usize) {
    //     self.slice_sorted = slice_sorted_move[N_MOVE * self.slice_sorted + m] as usize;
    //     self.corners = corners_move[N_MOVE * self.corners + m] as usize;
    //     self.ud_edges = ud_edges_move[N_MOVE * (self.ud_edges as usize) + m] as i32;
    // }

    /// Compute the depth of the phase 1 search.
    ///
    /// # Returns
    ///
    /// The depth of the phase 1 search.
    pub fn get_depth_phase1(&self) -> usize {
        let slice_ = self.slice_sorted / N_PERM_4;
        let flip = self.flip;
        let twist = self.twist;
        let flipslice = N_FLIP * slice_ + flip;
        let classidx = FLIPSLICE_CLASSIDX[flipslice];
        let sym = FLIPSLICE_SYM[flipslice];
        let depth_mod3 = get_flipslice_twist_depth3(
            N_TWIST * (classidx as usize) +
                (TWIST_CONJ[((twist << 4) + (sym as usize)) as usize] as usize)
        );

        let mut depth = 0;
        let mut flip_ = flip;
        let mut slice__ = slice_;
        let mut twist_ = twist;
        let mut depth_mod3_ = depth_mod3;

        while flip_ != SOLVED || slice__ != SOLVED || twist_ != SOLVED {
            if depth_mod3_ == 0 {
                depth_mod3_ = 3;
            }
            for m in Move::iter() {
                let twist1 = TWIST_MOVE[N_MOVE * twist_ + (*m as usize)];
                let flip1 = FLIP_MOVE[N_MOVE * flip_ + (*m as usize)];
                let slice1 =
                    SLICE_SORTED_MOVE[N_MOVE * slice__ * N_PERM_4 + (*m as usize)] /
                    (N_PERM_4 as u16);
                let flipslice1 = N_FLIP * (slice1 as usize) + (flip1 as usize);
                let classidx1 = FLIPSLICE_CLASSIDX[flipslice1];
                let sym1 = FLIPSLICE_SYM[flipslice1];
                if
                    get_flipslice_twist_depth3(
                        N_TWIST * (classidx1 as usize) +
                            (TWIST_CONJ[((twist1 << 4) + (sym1 as u16)) as usize] as usize)
                    ) == depth_mod3_ - 1
                {
                    depth += 1;
                    twist_ = twist1 as usize;
                    flip_ = flip1 as usize;
                    slice__ = slice1 as usize;
                    depth_mod3_ -= 1;
                    break;
                }
            }
        }
        depth
    }

    /// Compute the depth of the phase 2 search.
    ///
    /// # Arguments
    ///
    /// * `corners`: The permutation of the corner pieces.
    /// * `ud_edges`: The permutation of the up/down edge pieces.
    ///
    /// # Returns
    ///
    /// The depth of the phase 2 search.
    pub fn get_depth_phase2(corners: u32, ud_edges: u32) -> i8 {
        debug!("GET DEPTH PHASE 2: {}, {}", corners, ud_edges);

        let classidx = CORNER_CLASSIDX[corners as usize];
        let sym = CORNER_SYM[corners as usize];
        let mut depth_mod3 = get_corners_ud_edges_depth3(
            N_UD_EDGES * (classidx as usize) +
                (UD_EDGES_CONJ[((ud_edges << 4) + (sym as u32)) as usize] as usize)
        );

        debug!("Classidx: {}, Sym: {}, Depth mod 3: {}", classidx, sym, depth_mod3);

        if depth_mod3 == 3 {
            debug!("Return get depht phase 2\n");
            return 11;
        }

        let mut depth = 0;
        let mut corners = corners;
        let mut ud_edges = ud_edges;

        while corners != (SOLVED as u32) || ud_edges != (SOLVED as u32) {
            if depth_mod3 == 0 {
                depth_mod3 = 3;
            }

            // Seulement les mouvements de phase 2
            for m in &[
                Move::U1,
                Move::U2,
                Move::U3,
                Move::R2,
                Move::F2,
                Move::D1,
                Move::D2,
                Move::D3,
                Move::L2,
                Move::B2,
            ] {
                let corners1 = CORNERS_MOVE[(N_MOVE * (corners as usize) + (*m as usize)) as usize];
                let ud_edges1 =
                    UD_EDGES_MOVE[(N_MOVE * (ud_edges as usize) + (*m as usize)) as usize];
                let classidx1 = CORNER_CLASSIDX[corners1 as usize];
                let sym = CORNER_SYM[corners1 as usize];

                if
                    get_corners_ud_edges_depth3(
                        N_UD_EDGES * (classidx1 as usize) +
                            (UD_EDGES_CONJ[((ud_edges1 as usize) << 4) + (sym as usize)] as usize)
                    ) == depth_mod3 - 1
                {
                    debug!("Break");
                    depth += 1;
                    corners = corners1 as u32;
                    ud_edges = ud_edges1 as u32;
                    depth_mod3 -= 1;
                    break;
                }
            }
        }

        depth
    }
}
