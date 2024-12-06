//! This module handle the definition of [`CubieCube`].

// local
use crate::warning;
use super::{
    defs::*,
    enums::{ Corner, Edge, Move },
    face_cube::*,
    utils::{ c_nk, rotate_left, rotate_right },
};

// std
use rand::Rng;
use std::fmt;

/// Represents the state of a Rubik's Cube in terms of its cubies.
#[derive(Debug, Clone, Copy)]
pub struct CubieCube {
    /// The permutation of the corner cubies.
    pub cp: [Corner; 8],
    /// The orientation of the corner cubies.
    pub co: [u8; 8],
    /// The permutation of the edge cubies.
    pub ep: [Edge; 12],
    /// The orientation of the edge cubies.
    pub eo: [u8; 12],
}

impl CubieCube {
    /// Create a new Cube.
    /// # Args :
    /// * `cp` : The corners permutations (positions) of the cube.
    /// * `co` : The corners orientations of the cube.
    /// * `ep` : The edges permutations (positions) of the cube.
    /// * `eo` : The edges orientations of the cube.
    pub const fn new(cp: [Corner; 8], co: [u8; 8], ep: [Edge; 12], eo: [u8; 12]) -> Self {
        Self { cp, co, ep, eo }
    }

    /// Returns the corner permutations of the cube
    pub fn get_cp(&self) -> [Corner; 8] {
        self.cp
    }

    /// Returns the corner orientations of the cube
    pub fn get_co(&self) -> [u8; 8] {
        self.co
    }

    /// Returns the edge permutations of the cube
    pub fn get_ep(&self) -> [Edge; 12] {
        self.ep
    }

    /// Returns the edge orientations of the cube
    pub fn get_eo(&self) -> [u8; 12] {
        self.eo
    }

    /// Multiply this cubie cube with another cubie cube "other", restricted to the corners. Does not change other.
    pub fn corner_multiply(&mut self, other: CubieCube) {
        let mut new_corner_perm = [0; 8];
        let mut new_corner_ori = [0; 8];
        let mut current_ori = 0;

        for c in 0..N_CORNERS_ENUM {
            new_corner_perm[c] = self.cp[other.cp[c] as usize] as u8;
            let ori_self = self.co[other.cp[c] as usize] as i8;
            let ori_other = other.co[c] as i8;
            if ori_self < 3 && ori_other < 3 {
                // two regular cubes

                current_ori = ori_self + ori_other;
                if current_ori >= 3 {
                    current_ori -= 3;
                }
            } else if ori_self < 3 && ori_other >= 3 {
                // cube b is in a mirrored state
                //println!("not rotating");
                current_ori = ori_self + ori_other;
                if current_ori >= 6 {
                    current_ori -= 3;
                }
            } else if ori_self >= 3 && ori_other < 3 {
                // cube a is in a mirrored state
                //println!("not rotating");
                current_ori = ori_self - ori_other;
                if current_ori < 3 {
                    current_ori += 3;
                }
            } else if ori_self >= 3 && ori_other >= 3 {
                // if both cubes are in mirrored states
                //println!("not rotating");
                current_ori = ori_self - ori_other;
                if current_ori < 0 {
                    current_ori += 3;
                }
            }
            new_corner_ori[c] = current_ori;
        }

        for c in 0..N_CORNERS_ENUM {
            self.cp[c] = new_corner_perm[c].into();
            self.co[c] = new_corner_ori[c] as u8;
        }
    }

    /// Multiply this Cubie Cube with another Cubie Cube 'other', restricted to the edges. Does not change 'other'.
    pub fn edge_multiply(&mut self, other: CubieCube) {
        let mut new_edge_permutation: Vec<u8> = vec![0; 12];
        let mut new_edge_orientation: Vec<u8> = vec![0; 12];
        for edge in 0..12 {
            new_edge_permutation[edge] = self.ep[other.ep[edge] as usize] as u8;
            // the new edge orientation = other orientation + current orientation
            // we use other.ep[edge] rather than just edge to take in acount the permutation also !!
            new_edge_orientation[edge] = (other.eo[edge] + self.eo[other.ep[edge] as usize]) % 2;
        }
        // Update the edge permutation and orientation of the current cube
        for edge in 0..12 {
            self.ep[edge] = new_edge_permutation[edge].into();
            self.eo[edge] = new_edge_orientation[edge];
        }
    }

    ///apply the differente move of the cube other to current cube
    pub fn multiply(&mut self, other: CubieCube) {
        self.corner_multiply(other);
        self.edge_multiply(other);
    }

    /// """Store the inverse of this cubie cube in d."""
    pub fn inv_cubie_cube(&mut self, other: &mut CubieCube) {
        for e in Edge::iter() {
            other.ep[self.ep[e as usize] as usize] = e;
        }

        for e in Edge::iter() {
            other.eo[e as usize] = self.eo[other.ep[e as usize] as usize];
        }

        for c in Corner::iter() {
            other.cp[self.cp[c as usize] as usize] = c;
        }

        for c in Corner::iter() {
            let ori = self.co[other.cp[c as usize] as usize];
            other.co[c as usize] = ori;
            // impossible normalement
            // if ori >= 3 {
            //     other.co[c as usize] = ori;
            // } else {
            //
            //     other.co[c as usize] = -ori;
            //     if other.co[c as usize] < 0 {
            //         other.co[c as usize] += 3;
            //     }
            // }
        }
    }

    /// Returns the 'twist' of the cube, which represents the orientation of all its (8) corners
    /// as a single number between 0 and 2187.
    ///
    /// There are 3 orientations for each of the 8 corners, but we ignore one because it's not possible
    /// that only one corner is twisted. This gives us 3^7 = 2187 possibilities.
    ///
    /// This should not be confused with 'flip', which refers to the orientation of the edge cubies.
    ///
    /// # Returns
    ///
    /// The 'twist' of the cube.
    pub fn get_twist(self) -> usize {
        let mut ret = 0;
        // from 0 to 7 because of the rubik's cube law (it's not possible that only one corner is twisted)
        for i in 0..7 {
            ret = 3 * ret + (self.co[i] as usize);
        }
        ret
    }

    /// Sets the 'twist' of the cube based on the provided twist value.
    ///
    /// # Arguments
    ///
    /// * `twist`: The desired twist value to set.
    pub fn set_twist(&mut self, mut twist: usize) {
        let mut twist_parity = 0;
        for i in (0..7).rev() {
            self.co[i] = (twist % 3) as u8;
            twist_parity += self.co[i];
            twist /= 3;
        }
        self.co[7] = (3 - (twist_parity % 3)) % 3;
    }

    /// Returns the 'flip' of the cube, which represents the orientation of all its (12) edges
    /// as a single number between 0 and 2048.
    ///
    /// There are 2 orientations for each of the 12 edges, but we ignore one because it's not possible
    /// that only one edge is twisted. This gives us 2^11 = 2048 possibilities.
    ///
    /// This should not be confused with 'twist', which refers to the orientation of the corner cubies.
    ///
    /// # Returns
    ///
    /// The 'flip' of the cube.
    pub fn get_flip(self) -> usize {
        let mut ret = 0;
        for i in 0..11 {
            ret = 2 * ret + (self.eo[i] as usize);
        }
        ret
    }

    /// Sets the 'flip' of the cube based on the provided flip value.
    ///
    /// # Arguments
    ///
    /// * `flip`: The desired flip value to set.
    pub fn set_flip(&mut self, mut flip: usize) {
        let mut flip_parity = 0;
        for i in (0..11).rev() {
            self.eo[i] = (flip % 2) as u8;
            flip_parity += self.eo[i];
            flip /= 2;
        }
        self.eo[11] = (2 - (flip_parity % 2)) % 2;
    }

    /// Returns the index representing the location of the UD-slice edges FR, FL, BL, and BR
    /// ignoring their permutation.
    ///
    /// The returned index `a` satisfies the inequality 0 <= a < 495 in phase 1, and a = 0 in phase 2.
    ///
    /// # Returns
    ///
    /// The index representing the location of the UD-slice edges.
    pub fn get_slice(&self) -> usize {
        // Get the location of the UD-slice edges FR,FL,BL and BR ignoring their permutation.
        // 0<= slice < 495 in phase 1, slice = 0 in phase 2.

        let mut a = 0;
        let mut x = 0;

        // Compute the index a < (12 choose 4)
        for j in (Edge::BR as usize..=Edge::UR as usize).rev() {
            if
                (Edge::FR as usize) <= (self.ep[j] as usize) &&
                (self.ep[j] as usize) <= (Edge::BR as usize)
            {
                a += c_nk(11 - j, x + 1);
                x += 1;
            }
        }

        a
    }

    /// Sets the location of the UD-slice edges FR, FL, BL, and BR based on the provided index.
    ///
    /// This function sets the location of the UD-slice edges in the cube's edge permutation array based on the provided index.
    /// The provided index represents the location of the UD-slice edges and ignores their permutation.
    ///
    /// # Arguments
    ///
    /// * `idx`: The index representing the location of the UD-slice edges.
    fn _set_slice(&mut self, idx: usize) {
        let slice_edge = [
            Edge::FR as usize,
            Edge::FL as usize,
            Edge::BL as usize,
            Edge::BR as usize,
        ];
        let other_edge = [
            Edge::UR as usize,
            Edge::UF as usize,
            Edge::UL as usize,
            Edge::UB as usize,
            Edge::DR as usize,
            Edge::DF as usize,
            Edge::DL as usize,
            Edge::DB as usize,
        ];
        let mut a = idx; // Location

        for e in 0..12 {
            //self.ep[e] = -1; // Invalidate all edge positions
            self.ep[e] = Edge::INVALID;
        }

        let mut x = 4; // set slice edges
        for j in (Edge::BR as usize..=Edge::UR as usize).rev() {
            if a >= c_nk(11 - j, x) {
                self.ep[j] = slice_edge[4 - x].into();
                a -= c_nk(11 - j, x);
                x -= 1;
            }
        }

        x = 0; // set the remaining edges UR..DB
        for j in 0..12 {
            //if self.ep[j] == -1 {
            if self.ep[j] == Edge::INVALID {
                self.ep[j] = other_edge[x].into();
                x += 1;
            }
        }
    }

    /// Returns the index representing the location of the sorted UD-slice edges.
    ///
    /// The sorted UD-slice edges are determined by their permutation and orientation.
    /// This function computes an index `a` representing the position of the edges in the slice
    /// based on their permutation, and an index `b` representing the permutation of the edges within the slice.
    /// The final index is a combination of these two indices.
    ///
    /// # Returns
    ///
    /// The index representing the location of the sorted UD-slice edges.
    pub fn get_slice_sorted(&self) -> usize {
        let mut a = 0;
        let mut x = 0;
        let mut edge4: [usize; 4] = [0; 4];

        // First compute the index a < (12 choose 4) and the permutation array perm.
        for j in (0..12).rev() {
            if
                (self.ep[j] as usize) >= (Edge::FR as usize) &&
                (self.ep[j] as usize) <= (Edge::BR as usize)
            {
                a += c_nk(11 - j, x + 1);
                edge4[3 - x] = self.ep[j] as usize;
                x += 1;
            }
        }

        // Then compute the index b < 4! for the permutation in edge4
        let mut b = 0;
        for j in (1..4).rev() {
            let mut k = 0;
            while (edge4[j] as usize) != (j as usize) + 8 {
                rotate_left(&mut edge4, 0, j);
                k += 1;
            }
            b = (j + 1) * b + k;
        }

        24 * a + b
    }

    /// Sets the location and permutation of the sorted UD-slice edges based on the provided index.
    ///
    /// This function sets the location and permutation of the UD-slice edges in the cube's edge permutation array
    /// based on the provided index. The provided index represents the combined location and permutation of the edges
    /// within the slice.
    ///
    /// # Arguments
    ///
    /// * `idx`: The index representing the location and permutation of the sorted UD-slice edges.
    pub fn set_slice_sorted(&mut self, idx: usize) {
        let mut slice_edge = vec![Edge::FR, Edge::FL, Edge::BL, Edge::BR];
        let other_edge = vec![
            Edge::UR,
            Edge::UF,
            Edge::UL,
            Edge::UB,
            Edge::DR,
            Edge::DF,
            Edge::DL,
            Edge::DB
        ];
        let mut b = idx % 24; // Permutation
        let mut a = idx / 24; // Location
        for e in Edge::iter() {
            //self.ep[e as usize] = -1; // Invalidate all edge positions
            self.ep[e as usize] = Edge::INVALID; // Invalidate all edge positions
        }

        let mut j = 1; // generate permutation from index b
        while j < 4 {
            let mut k = b % (j + 1);
            b /= j + 1;
            while k > 0 {
                rotate_right(&mut slice_edge, 0, j);
                k -= 1;
            }
            j += 1;
        }

        let mut x = 4; // set slice edges
        for e in Edge::iter() {
            if a >= c_nk(11 - (e as usize), x) {
                self.ep[e as usize] = slice_edge[4 - x];
                a -= c_nk(11 - (e as usize), x);
                x -= 1;
            }
        }

        x = 0; // set the remaining edges UR..DB
        for e in Edge::iter() {
            //if self.ep[e as usize] == -1 {
            if self.ep[e as usize] == Edge::INVALID {
                self.ep[e as usize] = other_edge[x];
                x += 1;
            }
        }
    }

    /// Returns the index representing the location and permutation of the UD-slice edges UR, UF, UL, and UB.
    ///
    /// This function computes an index `a` representing the position of the edges in the UD-slice
    /// based on their permutation and an index `b` representing the permutation of the edges within the slice.
    /// The final index is a combination of these two indices.
    ///
    /// # Returns
    ///
    /// The index representing the location and permutation of the UD-slice edges UR, UF, UL, and UB.
    pub fn get_u_edges(&self) -> usize {
        let mut a = 0;
        let mut x = 0;
        let mut edge4 = [0; 4];
        let mut ep_mod = self.ep.clone();

        // Rotate ep_mod right by 3 positions
        for _ in 0..4 {
            rotate_right(&mut ep_mod, 0, 11);
        }

        // First compute the index a < (12 choose 4) and the permutation array perm.
        for j in (0..12).rev() {
            if
                (Edge::UR as usize) <= (ep_mod[j] as usize) &&
                (ep_mod[j] as usize) <= (Edge::UB as usize)
            {
                a += c_nk(11 - j, x + 1);
                edge4[3 - x] = ep_mod[j] as usize;
                x += 1;
            }
        }

        // Then compute the index b < 4! for the permutation in edge4
        let mut b = 0;
        for j in (1..4).rev() {
            let mut k = 0;
            while edge4[j] != j {
                rotate_left(&mut edge4, 0, j);
                k += 1;
            }
            b = (j + 1) * b + k;
        }

        24 * a + b
    }

    /// Sets the location and permutation of the UD-slice edges UR, UF, UL, and UB based on the provided index.
    ///
    /// This function sets the location and permutation of the UD-slice edges in the cube's edge permutation array
    /// based on the provided index. The provided index represents the combined location and permutation of the edges
    /// within the slice.
    ///
    /// # Arguments
    ///
    /// * `idx`: The index representing the location and permutation of the UD-slice edges UR, UF, UL, and UB.
    pub fn set_u_edges(&mut self, idx: usize) {
        let mut slice_edge = vec![Edge::UR, Edge::UF, Edge::UL, Edge::UB];
        let other_edge = vec![
            Edge::DR,
            Edge::DF,
            Edge::DL,
            Edge::DB,
            Edge::FR,
            Edge::FL,
            Edge::BL,
            Edge::BR
        ];
        let mut b = idx % 24; // Permutation
        let mut a = idx / 24; // Location

        for e in &mut self.ep {
            *e = Edge::INVALID; // Invalidate all edge positions
        }

        let mut j = 1; // generate permutation from index b
        while j < 4 {
            let mut k = b % (j + 1);
            b /= j + 1;
            while k > 0 {
                rotate_right(&mut slice_edge, 0, j);
                k -= 1;
            }
            j += 1;
        }

        let mut x = 4; // set slice edges
        for j in (0..12).rev() {
            if a >= c_nk(11 - j, x) {
                self.ep[j] = slice_edge[4 - x];
                a -= c_nk(11 - j, x);
                x -= 1;
            }
        }

        let mut x = 0; // set the remaining edges UR..DB
        for j in 0..12 {
            if self.ep[j] == Edge::INVALID {
                self.ep[j] = other_edge[x];
                x += 1;
            }
        }

        for _ in 0..4 {
            rotate_left(&mut self.ep, 0, 11);
        }
    }

    /// Returns the index representing the location and permutation of the D-slice edges DR, DF, DL, and DB.
    ///
    /// This function computes an index `a` representing the position of the edges in the D-slice
    /// based on their permutation and an index `b` representing the permutation of the edges within the slice.
    /// The final index is a combination of these two indices.
    ///
    /// # Returns
    ///
    /// The index representing the location and permutation of the D-slice edges DR, DF, DL, and DB.
    pub fn get_d_edges(&self) -> usize {
        let mut a = 0;
        let mut x = 0;
        let mut edge4 = vec![0; 4];
        let mut ep_mod = self.ep.clone();

        for _ in 0..4 {
            rotate_right(&mut ep_mod, 0, 11);
        }

        // First compute the index a < (12 choose 4) and the permutation array perm.
        for j in (0..12).rev() {
            if
                (Edge::DR as usize) <= (ep_mod[j] as usize) &&
                (ep_mod[j] as usize) <= (Edge::DB as usize)
            {
                a += c_nk(11 - j, x + 1);
                edge4[3 - x] = ep_mod[j] as usize;
                x += 1;
            }
        }

        // Then compute the index b < 4! for the permutation in edge4
        let mut b = 0;
        for j in (1..=3).rev() {
            let mut k = 0;
            while edge4[j] != (j as usize) + 4 {
                rotate_left(&mut edge4, 0, j);
                k += 1;
            }
            b = (j + 1) * b + k;
        }

        24 * a + b
    }

    /// Sets the location and permutation of the D-slice edges DR, DF, DL, and DB based on the provided index.
    ///
    /// This function sets the location and permutation of the D-slice edges in the cube's edge permutation array
    /// based on the provided index. The provided index represents the combined location and permutation of the edges
    /// within the D-slice.
    ///
    /// # Arguments
    ///
    /// * `idx`: The index representing the location and permutation of the D-slice edges DR, DF, DL, and DB.
    pub fn set_d_edges(&mut self, idx: usize) {
        let mut slice_edge = vec![Edge::DR, Edge::DF, Edge::DL, Edge::DB];
        let other_edge = vec![
            Edge::FR,
            Edge::FL,
            Edge::BL,
            Edge::BR,
            Edge::UR,
            Edge::UF,
            Edge::UL,
            Edge::UB
        ];
        let mut b = idx % 24; // Permutation
        let mut a = idx / 24; // Location

        // Invalidate all edge positions
        for e in Edge::UR as usize..=Edge::BR as usize {
            //self.ep[e] = -1;
            self.ep[e] = Edge::INVALID;
        }

        let mut j = 1; // generate permutation from index b
        while j < 4 {
            let mut k = b % (j + 1);
            b /= j + 1;
            while k > 0 {
                rotate_right(&mut slice_edge, 0, j);
                k -= 1;
            }
            j += 1;
        }

        let mut x = 4; // set slice edges
        for j in Edge::UR as usize..=Edge::BR as usize {
            if (a as i64) - (c_nk(11 - j, x) as i64) <= 0 {
                self.ep[j] = slice_edge[4 - x];
                a -= c_nk(11 - j, x);
                x -= 1;
            }
        }

        x = 0; // set the remaining edges UR..DB
        for j in Edge::UR as usize..=Edge::BR as usize {
            //if self.ep[j] == -1 {
            if self.ep[j] == Edge::INVALID {
                self.ep[j] = other_edge[x];
                x += 1;
            }
        }

        for _ in 0..4 {
            rotate_left(&mut self.ep, 0, 11);
        }
    }

    /// Returns the index representing the permutation of the corner pieces.
    ///
    /// This function computes an index representing the permutation of the corner pieces in the cube's corner permutation array.
    ///
    /// # Returns
    ///
    /// The index representing the permutation of the corner pieces.
    pub fn get_corners(&self) -> usize {
        let mut perm = self.cp.clone(); // duplicate cp
        let mut b = 0;
        for j in (0..8).rev() {
            let mut k = 0;
            while perm[j] != j.into() {
                rotate_left(&mut perm, 0, j);
                k += 1;
            }
            b = (j + 1) * b + k;
        }
        b
    }

    /// Sets the permutation of the corner pieces based on the provided index.
    ///
    /// This function sets the permutation of the corner pieces in the cube's corner permutation array
    /// based on the provided index.
    ///
    /// # Arguments
    ///
    /// * `idx`: The index representing the permutation of the corner pieces.
    pub fn set_corners(&mut self, mut idx: usize) {
        self.cp = [
            Corner::URF,
            Corner::UFL,
            Corner::ULB,
            Corner::UBR,
            Corner::DFR,
            Corner::DLF,
            Corner::DBL,
            Corner::DRB,
        ];
        for j in Corner::URF as usize..=Corner::DRB as usize {
            let mut k = idx % (j + 1);
            idx /= j + 1;
            while k > 0 {
                rotate_right(&mut self.cp, 0, j);
                k -= 1;
            }
        }
    }

    pub fn get_ud_edges(&self) -> usize {
        let mut perm = self.ep[..8].to_vec(); // duplicate first 8 elements of ep
        let mut b = 0;
        for j in Edge::DB as usize..=Edge::UR as usize {
            let mut k = 0;
            while perm[j] != j.into() {
                rotate_left(&mut perm, 0, j);
                k += 1;
            }
            b = (j + 1) * b + k;
        }
        b
    }

    pub fn set_ud_edges(&mut self, mut idx: usize) {
        // positions of FR FL BL BR edges are not affected
        for i in Edge::UR as usize..=Edge::BR as usize {
            self.ep[i] = i.into();
        }
        for j in Edge::UR as usize..=Edge::DB as usize {
            let mut k = idx % (j + 1);
            idx /= j + 1; // pas sur
            while k > 0 {
                rotate_right(&mut self.ep, 0, j);
                k -= 1;
            }
        }
    }

    /// Returns a facelet representation of the cube.
    ///
    /// This function converts the current cube representation into a FaceCube, which is a facelet-based representation
    /// of the Rubik's Cube. Each facelet of the cube is mapped to its corresponding color and set in the FaceCube.
    ///
    /// # Returns
    ///
    /// A FaceCube representation of the cube.
    pub fn to_facelet_cube(self) -> FaceCube {
        let mut face_cube = FaceCube::new();
        // handle corners
        for i in 0..N_CORNERS_ENUM {
            let corner = self.cp[i as usize]; // corner j is at corner position i
            let orientation = self.co[i as usize]; // orientation of C j at position i
            for k in 0..3 {
                // get the corresponding facelet in the transition list between a corner and the facelets
                let index = CORNER_FACELET[i as usize][((k + orientation) % 3) as usize] as usize;
                // get the corresponding color
                // WARNING: why is there not ((k + orientation) % 3) as usize???
                let new_color = CORNER_COLOR[corner as usize][k as usize];
                face_cube.set_facelet(index, new_color);
            }
        }

        // handle edges
        for i in 0..N_EDGES {
            let edges = self.ep[i as usize]; // similar for Es
            let orientation = self.eo[i as usize];
            for k in 0..2 {
                let index = EDGE_FACELET[i as usize][((k + orientation) % 2) as usize] as usize;
                let new_color = EDGE_COLOR[edges as usize][k as usize];
                face_cube.set_facelet(index, new_color);
            }
        }
        face_cube
    }

    /// Returns the parity of the corner permutation.
    ///
    /// This function calculates the parity of the corner permutation of the cube.
    ///
    /// # Returns
    ///
    /// The parity of the corner permutation, which is either 0 or 1.
    pub fn corner_parity(self) -> u8 {
        let mut parity = 0;
        for i in (0..N_CORNERS_ENUM).rev() {
            for permutation in (0..=i).rev() {
                if (self.cp[permutation as usize] as u8) > (self.cp[i as usize] as u8) {
                    parity += 1;
                }
            }
        }
        parity % 2
    }

    /// Returns the parity of the edge permutation.
    ///
    /// This function calculates the parity of the edge permutation of the cube.
    ///
    /// # Returns
    ///
    /// The parity of the edge permutation, which is either 0 or 1.
    ///
    /// A solvable cube has the same corner and edge parity.
    pub fn edge_parity(self) -> u8 {
        let mut parity = 0;
        for edge in (0..N_EDGES).rev() {
            for permutation in (0..=edge).rev() {
                if (self.ep[permutation as usize] as u8) > (self.ep[edge as usize] as u8) {
                    parity += 1;
                }
            }
        }
        parity % 2
    }

    /// Returns the symmetries of the cube.
    ///
    /// This function calculates and returns the symmetries of the cube, which are represented as a vector of integers.
    ///
    /// # Returns
    ///
    /// A vector containing the symmetries of the cube.
    pub fn symmetries(&self) -> Vec<usize> {
        let mut s = vec![];
        let mut d = DEFAULT;

        for j in 0..N_SYM {
            let mut c = SYM_CUBE[j];
            c.multiply(*self);
            c.multiply(SYM_CUBE[INV_IDX[j] as usize]);
            if *self == c {
                s.push(j);
            }
            c.inv_cubie_cube(&mut d);
            if *self == d {
                s.push(j + N_SYM);
            }
        }
        s
    }

    /// Verifies if the CubieCube is valid.
    ///
    /// This function checks if the CubieCube represents a valid Rubik's Cube configuration.
    /// It verifies the edge and corner pieces, their orientations, as well as the edge and corner parity.
    ///
    /// # Returns
    ///
    /// Returns `true` if the CubieCube is valid, otherwise returns `false`.
    pub fn verify(&self) -> bool {
        let mut edge_count = [0; 12];
        for i in 0..N_EDGES {
            edge_count[self.ep[i as usize] as usize] += 1;
        }
        for edge in 0..N_EDGES {
            if edge_count[edge as usize] != 1 {
                warning!("Some edges are invalid ...");
                return false;
            }
        }

        let mut verify_edge = 0;
        for edge in 0..N_EDGES {
            verify_edge += self.eo[edge as usize];
        }
        if verify_edge % 2 != 0 {
            warning!("Total edge flip is wrong ...");
            return false;
        }

        let mut corner_count = [0; 8];
        for corner in 0..N_CORNERS_ENUM {
            corner_count[self.cp[corner as usize] as usize] += 1;
        }
        for corner in 0..N_CORNERS_ENUM {
            if corner_count[corner as usize] != 1 {
                warning!("Some corners are invalid ...");
                return false;
            }
        }

        let mut verify_corner = 0;

        for corner in 0..N_CORNERS_ENUM {
            verify_corner += self.co[corner as usize];
        }

        if verify_corner % 3 != 0 {
            warning!("Total corner flip is wrong...");
            return false;
        }

        if self.edge_parity() != self.corner_parity() {
            warning!("Wrong egde and corner parity...");
            return false;
        }

        return true;
    }

    /// Randomly shuffle the [`CubieCube`] by applying it 100 random moves.
    pub fn randomize(&mut self, n: u32) -> Vec<Move> {
        let mut res = vec![];

        let mut rng = rand::thread_rng();
        for _ in 0..n {
            let n = rng.gen_range(0..N_MOVES);

            self.multiply(ALL_MOVES[n]);

            res.push(Move::from(n));
        }

        res
    }
}

/// Display implementation for CubieCube (to_string() method)
///
/// ## Format :
///
/// (corner, orientation) ... (edge, orientation)
impl fmt::Display for CubieCube {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut string = String::new();
        string += "|";
        for edge in 0..12 {
            // The 12 edges
            string += format!("({},{})", self.ep[edge], self.eo[edge]).as_str();
        }
        for corner in 0..8 {
            string += format!("({},{})", self.cp[corner], self.co[corner]).as_str();

            let f = match self.cp[corner] {
                Corner::URF => "UFR",
                Corner::UFL => "ULF",
                Corner::ULB => "UBL",
                Corner::UBR => "URB",
                Corner::DFR => "DRF",
                Corner::DLF => "DFL",
                Corner::DBL => "DLB",
                Corner::DRB => "DBR",
            };

            string += format!("{} ", f).as_str();
        }
        write!(f, "{}", string)
    }
}

/// Compare row CubieCube.
///
/// # Example
///
/// ```
/// let cube_a = CubieCube::new();
/// let cube_b = CubieCube::new();
/// if cube_a == cube_b {
///     // Do something
/// }
/// ```
impl PartialEq for CubieCube {
    fn eq(&self, other: &Self) -> bool {
        self.cp == other.cp && self.co == other.co && self.ep == other.ep && self.eo == other.eo
    }
}
