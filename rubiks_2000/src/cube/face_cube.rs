//! This module handle the [`FaceCube`] definition.

use super::{ cubie_cube::*, defs::*, enums::* };

//The names of the facelet positions of the cube
//               |*************|
//               |* 0 * 1 * 2 *|
//               |*************|
//               |* 3 * 4 * 5 *|
//               |*************|
//               |* 6 * 7 * 8 *|
//               |*************|
// |*************|*************|*************|*************|
// |*36 *37 *38 *|*18 *19 *20 *|* 9 *10 *11 *|*45 *46 *47 *|
// |*************|*************|*************|*************|
// |*39 *40 *41 *|*21 *22 *23 *|*12 *13 *14 *|*48 *49 *50 *|
// |*************|*************|*************|*************|
// |*42 *43 *44 *|*24 *25 *26 *|*15 *16 *17 *|*51 *52 *53 *|
// |*************|*************|*************|*************|
//               |*************|
//               |*27 *28 *29 *|
//               |*************|
//               |*30 *31 *32 *|
//               |*************|
//               |*33 *34 *35 *|
//               |*************|

/// Represents the facelet colors of a Rubik's Cube.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FaceCube {
    /// An array storing the color tags of the facelets.
    facelets: [ColorTag; 54],
}

impl FaceCube {
    /// Creat an empty [`FaceCube`] filled with `Empty` [`ColorTag`].
    pub const fn empty() -> Self {
        Self { facelets: [ColorTag::None; 54] }
    }

    /// Creates a new `FaceCube` with all facelets initialized to their default solved positions.
    pub fn new() -> Self {
        let mut facelets = [ColorTag::B; 54];
        // Assign colors to the facelets based on their position in the solve cube
        for up in 0..9 {
            facelets[up] = ColorTag::U;
        }
        for right in 9..18 {
            facelets[right] = ColorTag::R;
        }
        for front in 18..27 {
            facelets[front] = ColorTag::F;
        }
        for down in 27..36 {
            facelets[down] = ColorTag::D;
        }
        for left in 36..45 {
            facelets[left] = ColorTag::L;
        }
        FaceCube { facelets }
    }

    pub fn verify(self) -> bool {
        let mut count = [0; 6];

        for f in self.facelets {
            count[f as usize] += 1;
        }

        for c in count {
            if c != 9 {
                return false;
            }
        }

        return true;
    }

    /// Creates a new `FaceCube` from a string representation of the cube.
    pub fn from_string(str_cube: &str) -> Self {
        if str_cube.len() < 54 {
            panic!("pas le bon nombre de facelets");
        }
        if str_cube.len() > 54 {
            panic!("pas le bon nombre de facelets");
        }

        let mut facelets = [ColorTag::B; 54];
        let mut cpt = [0; 6];
        for (index, color) in str_cube.char_indices() {
            match color {
                'U' => {
                    facelets[index] = ColorTag::U;
                    cpt[0] += 1;
                }
                'B' => {
                    facelets[index] = ColorTag::B;
                    cpt[1] += 1;
                }
                'F' => {
                    facelets[index] = ColorTag::F;
                    cpt[2] += 1;
                }
                'D' => {
                    facelets[index] = ColorTag::D;
                    cpt[3] += 1;
                }
                'L' => {
                    facelets[index] = ColorTag::L;
                    cpt[4] += 1;
                }
                'R' => {
                    facelets[index] = ColorTag::R;
                    cpt[5] += 1;
                }
                _ => panic!("pas les bonnes lettres"),
            }
        }
        // verify if the correct number of each color was provide
        for i in 0..6 {
            if cpt[i] != 9 {
                panic!("probleme de couleur");
            }
        }
        FaceCube { facelets }
    }

    /// Converts the `FaceCube` into its string representation.
    /// Each facelet color is represented by a single character: 'U' for Up, 'R' for Right, 'F' for Front,
    /// 'D' for Down, 'L' for Left, and 'B' for Back.
    ///
    /// # Panics
    /// Panics if the cube contains empty facelets.
    pub fn to_string(self) -> String {
        let mut res = String::new();
        for i in 0..54 {
            match self.facelets[i] {
                ColorTag::B => res.push('B'),
                ColorTag::R => res.push('R'),
                ColorTag::U => res.push('U'),
                ColorTag::F => res.push('F'),
                ColorTag::L => res.push('L'),
                ColorTag::D => res.push('D'),
                ColorTag::None =>
                    panic!(
                        "Attempt to convert a non valable cube to string: cube contains empty facelets"
                    ),
            }
        }
        res
    }

    /// Converts the `FaceCube` into a `CubieCube` by extracting the corner and edge information,
    /// along with their positions and orientations.
    ///
    /// # Returns
    /// A `CubieCube` representing the same cube state as the `FaceCube`.
    pub fn to_cubie_cube(self) -> CubieCube {
        let mut cp = [Corner::URF; 8];
        let mut co: [u8; 8] = [1; 8];
        let mut ep = [Edge::UB; 12];
        let mut eo = [1; 12];
        for i in 0..N_CORNERS_ENUM {
            let fac = CORNER_FACELET[i as usize];
            let mut ori = 0;
            for diff_ori in 0..3 {
                if
                    self.facelets[fac[diff_ori] as usize] == ColorTag::U ||
                    self.facelets[fac[diff_ori] as usize] == ColorTag::D
                {
                    ori = diff_ori;
                    break;
                }
            }
            let col1 = self.facelets[fac[(ori + 1) % 3] as usize];
            let col2 = self.facelets[fac[(ori + 2) % 3] as usize];
            for j in 0..N_CORNERS_ENUM {
                let col = CORNER_COLOR[j as usize];
                if col1 == col[1] && col2 == col[2] {
                    cp[i as usize] = Corner::from(j);
                    co[i as usize] = ori as u8;
                    break;
                }
            }
        }
        for i in 0..N_EDGES {
            for j in 0..N_EDGES {
                if
                    self.facelets[EDGE_FACELET[i as usize][0] as usize] ==
                        EDGE_COLOR[j as usize][0] &&
                    self.facelets[EDGE_FACELET[i as usize][1] as usize] == EDGE_COLOR[j as usize][1]
                {
                    ep[i as usize] = Edge::from(j);
                    eo[i as usize] = 0;
                    break;
                }
                if
                    self.facelets[EDGE_FACELET[i as usize][0] as usize] ==
                        EDGE_COLOR[j as usize][1] &&
                    self.facelets[EDGE_FACELET[i as usize][1] as usize] == EDGE_COLOR[j as usize][0]
                {
                    ep[i as usize] = Edge::from(j);
                    eo[i as usize] = 1;
                    break;
                }
            }
        }
        CubieCube::new(cp, co, ep, eo)
    }

    /// Sets the color of a specific facelet in the `FaceCube`.
    ///
    /// # Arguments
    /// * `index` - The index of the facelet to set.
    /// * `new_color` - The new color to assign to the facelet.
    pub fn set_facelet(&mut self, index: usize, new_color: ColorTag) {
        self.facelets[index] = new_color;
    }

    /// Returns an array containing the colors of all facelets in the `FaceCube`.
    ///
    /// # Returns
    /// An array containing the colors of all facelets in the `FaceCube`.
    pub fn facelets(&self) -> [ColorTag; 54] {
        self.facelets
    }

    /// Prints a debug representation of the `FaceCube`.
    ///
    /// This method prints each face of the cube along with its color configuration.
    pub fn dbg(&self) {
        let face_names: [&str; 6] = ["UP", "RIGHT", "FRONT", "DOWN", "LEFT", "BACK"];

        for (i, name) in face_names.into_iter().enumerate() {
            println!("\nFace {} :", name);
            for j in 0..9 {
                if j != 0 && j % 3 == 0 {
                    println!();
                }
                print!("{}", self.facelets[i * 9 + j]);
            }
        }
    }

    /// la fonction faite 2h avant la soutenance
    pub fn reverse(&self) -> FaceCube{
        let mut f = FaceCube::new();

        let mat = [
            0, 3, 6, 1, 4, 7, 2, 5, 8, 20, 19, 18, 23, 22, 21, 26, 25, 24, 11, 10, 9, 14, 13,
            12, 17, 16, 15, 35, 32, 29, 34, 31, 28, 33, 30, 27, 47, 46, 45, 50, 49, 48, 53, 52,
            51, 38, 37, 36, 41, 40, 39, 44, 43, 42,
        ];

        for i in 0..54 {
            f.set_facelet(i, self.facelets()[mat[i]]);
        }

        for i in 0..54 {
            let fac = f.facelets()[i];

            if fac == ColorTag::R {
                f.set_facelet(i, ColorTag::F)
            }
            if fac == ColorTag::F {
                f.set_facelet(i, ColorTag::R)
            }
            if fac == ColorTag::L {
                f.set_facelet(i, ColorTag::B)
            }
            if fac == ColorTag::B {
                f.set_facelet(i, ColorTag::L)
            }
        }
        f
    }
}
