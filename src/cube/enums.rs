//! This module handle all the enum definition of the cube module.

// std
use std::{ fmt, slice::Iter };

// local
use crate::{ critical, Parameters };
use super::{ cubie_cube::CubieCube, defs::ALL_MOVES };

/// Represents the names of the facelet positions of the cube.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Facelet {
    //The names of the facelet positions of the cube
    //              |************|
    //              |*U1**U2**U3*|
    //              |************|
    //              |*U4**U5**U6*|
    //              |************|
    //              |*U7**U8**U9*|
    //              |************|
    // |************|************|************|************|
    // |*L1**L2**L3*|*F1**F2**F3*|*R1**R2**R3*|*B1**B2**B3*|
    // |************|************|************|************|
    // |*L4**L5**L6*|*F4**F5**F6*|*R4**R5**R6*|*B4**B5**B6*|
    // |************|************|************|************|
    // |*L7**L8**L9*|*F7**F8**F9*|*R7**R8**R9*|*B7**B8**B9*|
    // |************|************|************|************|
    //              |************|
    //              |*D1**D2**D3*|
    //              |************|
    //              |*D4**D5**D6*|
    //              |************|
    //              |*D7**D8**D9*|
    //              |************|
    //A cube definition string "UBL..." means for example: In position U1 we have the U-color, in position U2 we have the
    //B-color, in position U3 we have the L color etc. according to the order U1, U2, U3, U4, U5, U6, U7, U8, U9, R1, R2,
    //R3, R4, R5, R6, R7, R8, R9, F1, F2, F3, F4, F5, F6, F7, F8, F9, D1, D2, D3, D4, D5, D6, D7, D8, D9, L1, L2, L3, L4,
    //L5, L6, L7, L8, L9, B1, B2, B3, B4, B5, B6, B7, B8, B9 of the enum constants.
    U1 = 0,
    U2 = 1,
    U3 = 2,
    U4 = 3,
    U5 = 4,
    U6 = 5,
    U7 = 6,
    U8 = 7,
    U9 = 8,
    R1 = 9,
    R2 = 10,
    R3 = 11,
    R4 = 12,
    R5 = 13,
    R6 = 14,
    R7 = 15,
    R8 = 16,
    R9 = 17,
    F1 = 18,
    F2 = 19,
    F3 = 20,
    F4 = 21,
    F5 = 22,
    F6 = 23,
    F7 = 24,
    F8 = 25,
    F9 = 26,
    D1 = 27,
    D2 = 28,
    D3 = 29,
    D4 = 30,
    D5 = 31,
    D6 = 32,
    D7 = 33,
    D8 = 34,
    D9 = 35,
    L1 = 36,
    L2 = 37,
    L3 = 38,
    L4 = 39,
    L5 = 40,
    L6 = 41,
    L7 = 42,
    L8 = 43,
    L9 = 44,
    B1 = 45,
    B2 = 46,
    B3 = 47,
    B4 = 48,
    B5 = 49,
    B6 = 50,
    B7 = 51,
    B8 = 52,
    B9 = 53,
}

/// Represents the colors of the cube facelets.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ColorTag {
    /// Color of the Up (U) face.
    U = 0,
    /// Color of the Right (R) face.
    R = 1,
    /// Color of the Front (F) face.
    F = 2,
    /// Color of the Down (D) face.
    D = 3,
    /// Color of the Left (L) face.
    L = 4,
    /// Color of the Back (B) face.
    B = 5,
    /// No color assigned.
    None = 6,
}

impl ColorTag {
    /// Converts the color to its RGB representation.
    pub fn to_rgb_normalized(self) -> (f32, f32, f32) {
        if Parameters::use_default_color_table() {
            return Parameters::default_color_from_tag(self).rgb_normalized();
        } else {
            return Parameters::color_from_tag(self).rgb_normalized();
        }
    }

    //pub fn to_rgb_with_data(self, data: ColorData) -> RgbColor {
    //    if (self as u8) < 6 {
    //        return data.colors[self as usize];
    //    }
    //    RgbColor::new(0, 0, 0)
    //}

    /// Returns an iterator over all possible colors of the cube facelets.
    pub fn iter() -> impl Iterator<Item = ColorTag> {
        static COLORS: [ColorTag; 7] = [
            ColorTag::U,
            ColorTag::R,
            ColorTag::F,
            ColorTag::D,
            ColorTag::L,
            ColorTag::B,
            ColorTag::None,
        ];
        COLORS.iter().copied()
    }
}

impl fmt::Display for ColorTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<usize> for ColorTag {
    fn from(value: usize) -> Self {
        match value {
            0 => ColorTag::U,
            1 => ColorTag::R,
            2 => ColorTag::F,
            3 => ColorTag::D,
            4 => ColorTag::L,
            5 => ColorTag::B,
            _ => critical!("Invalid value for ColorTag: {}", value),
        }
    }
}

/// The names of the corner positions of the cube.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Corner {
    /// The corner with U(p), R(ight), and F(ront) facelets.
    URF = 0,
    /// The corner with U(p), F(ront), and L(eft) facelets.
    UFL = 1,
    /// The corner with U(p), L(eft), and B(ack) facelets.
    ULB = 2,
    /// The corner with U(p), B(ack), and R(ight) facelets.
    UBR = 3,
    /// The corner with D(own), F(ront), and R(ight) facelets.
    DFR = 4,
    /// The corner with D(own), L(eft), and F(ront) facelets.
    DLF = 5,
    /// The corner with D(own), B(ack), and L(eft) facelets.
    DBL = 6,
    /// The corner with D(own), R(ight), and B(ack) facelets.
    DRB = 7,
}

impl From<u8> for Corner {
    // Implementation of the `From<u8>` trait for the `Corner` enum.
    fn from(value: u8) -> Corner {
        u8_to_corner(value)
    }
}

impl From<usize> for Corner {
    // Implementation of the `From<usize>` trait for the `Corner` enum.
    fn from(value: usize) -> Corner {
        u8_to_corner(value as u8)
    }
}

/// Converts an unsigned 8-bit integer into a `Corner` enum variant.
fn u8_to_corner(value: u8) -> Corner {
    match value {
        0 => Corner::URF,
        1 => Corner::UFL,
        2 => Corner::ULB,
        3 => Corner::UBR,
        4 => Corner::DFR,
        5 => Corner::DLF,
        6 => Corner::DBL,
        7 => Corner::DRB,
        _ => panic!(), //critical!("Not handled value to convert into corner : {}", value),
    }
}

impl Corner {
    /// Returns an iterator over all corner positions of the cube.
    pub fn iter() -> impl Iterator<Item = Corner> {
        static VARIANTS: [Corner; 8] = [
            Corner::URF,
            Corner::UFL,
            Corner::ULB,
            Corner::UBR,
            Corner::DFR,
            Corner::DLF,
            Corner::DBL,
            Corner::DRB,
        ];

        VARIANTS.iter().copied()
    }
}

impl fmt::Display for Corner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use a match to associate each variant with its corresponding string
        let corner_str = match *self {
            Corner::URF => "URF",
            Corner::UFL => "UFL",
            Corner::ULB => "ULB",
            Corner::UBR => "UBR",
            Corner::DFR => "DFR",
            Corner::DLF => "DLF",
            Corner::DBL => "DBL",
            Corner::DRB => "DRB",
        };
        // Write the string into the Formatter
        write!(f, "{}", corner_str)
    }
}
/// The names of the edge positions of the cube. Edge UR e.g. has an U(p) and R(ight) facelet.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Edge {
    UR = 0,
    UF = 1,
    UL = 2,
    UB = 3,
    DR = 4,
    DF = 5,
    DL = 6,
    DB = 7,
    FR = 8,
    FL = 9,
    BL = 10,
    BR = 11,
    INVALID = 13,
}

impl Edge {
    /// Returns an iterator over the variants of the Edge enum.
    pub fn iter() -> impl Iterator<Item = Edge> {
        static VARIANTS: [Edge; 12] = [
            Edge::UR,
            Edge::UF,
            Edge::UL,
            Edge::UB,
            Edge::DR,
            Edge::DF,
            Edge::DL,
            Edge::DB,
            Edge::FR,
            Edge::FL,
            Edge::BL,
            Edge::BR,
            //Edge::INVALID,
        ];

        VARIANTS.iter().copied()
    }
}

// Implementation of the `From<u8>` trait for the `Edge` enum.
impl From<u8> for Edge {
    fn from(value: u8) -> Self {
        u8_to_edge(value)
    }
}

// Implementation of the `From<usize>` trait for the `Edge` enum.
impl From<usize> for Edge {
    fn from(value: usize) -> Self {
        u8_to_edge(value as u8)
    }
}

/// Converts an unsigned 8-bit integer into a `Edge` enum variant.
fn u8_to_edge(value: u8) -> Edge {
    match value {
        0 => Edge::UR,
        1 => Edge::UF,
        2 => Edge::UL,
        3 => Edge::UB,
        4 => Edge::DR,
        5 => Edge::DF,
        6 => Edge::DL,
        7 => Edge::DB,
        8 => Edge::FR,
        9 => Edge::FL,
        10 => Edge::BL,
        11 => Edge::BR,
        _ => panic!(), //critical!("Not handled value to convert in edge : {}", value),
    }
}

impl fmt::Display for Edge {
    // Implementation of the `fmt::Display` trait for the `Edge` enum.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Formats the value using the given formatter.
        let name = match self {
            Edge::UR => "UR",
            Edge::UF => "UF",
            Edge::UL => "UL",
            Edge::UB => "UB",
            Edge::DR => "DR",
            Edge::DF => "DF",
            Edge::DL => "DL",
            Edge::DB => "DB",
            Edge::FR => "FR",
            Edge::FL => "FL",
            Edge::BL => "BL",
            Edge::BR => "BR",
            Edge::INVALID => "INVALID",
        };
        write!(f, "{}", name)
    }
}

/// The moves in the face turn metric.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Move {
    //The moves in the faceturn metric. Not to be confused with the names of the facelet positions in class Facelet.
    U1 = 0,
    U2 = 1,
    U3 = 2,
    R1 = 3,
    R2 = 4,
    R3 = 5,
    F1 = 6,
    F2 = 7,
    F3 = 8,
    D1 = 9,
    D2 = 10,
    D3 = 11,
    L1 = 12,
    L2 = 13,
    L3 = 14,
    B1 = 15,
    B2 = 16,
    B3 = 17,
    Default,
}

impl Move {
    /// Get the corresponding CubieCube for the move.
    pub fn get_cube(self) -> CubieCube {
        let n = self as usize;
        ALL_MOVES[n]
    }

    /// Checks if two moves belong to the same slice.
    pub fn is_same_slice(self, other: Move) -> bool {
        let s = self as u8;
        let m = s % 3;
        let o = other as u8;
        match m {
            0 => o >= s && o < s + 3,
            1 => o >= s - 1 && o <= s + 1,
            _ => o >= s - 2 && o <= s,
        }
    }

    /// Checks if two moves are on opposing slices.
    pub fn is_opposed_slice(self, other: Move) -> bool {
        ((self as u8) % 9) / 3 == ((other as u8) % 9) / 3
    }

    /// Returns the inverse of the given move.
    pub fn inverse(self) -> Move {
        let mut res = self as usize;
        let temp = res % 3;
        match temp {
            0 => {
                res += 2;
            }
            2 => {
                res -= 2;
            }
            _ => (),
        }
        Move::from(res)
    }

    pub fn reduce(self) -> Move {
        let mut res = self as usize;
        if res % 3 == 1 {
            res -= 1;
        }
        Move::from(res)
    }

    /// Returns an iterator over all possible moves.
    pub fn iter() -> Iter<'static, Move> {
        static MOVES: [Move; 18] = [
            Move::U1,
            Move::U2,
            Move::U3,
            Move::R1,
            Move::R2,
            Move::R3,
            Move::F1,
            Move::F2,
            Move::F3,
            Move::D1,
            Move::D2,
            Move::D3,
            Move::L1,
            Move::L2,
            Move::L3,
            Move::B1,
            Move::B2,
            Move::B3,
        ];
        MOVES.iter()
    }
}

// Implementation of the `From<usize>` trait for the `Move` enum.
impl From<usize> for Move {
    fn from(value: usize) -> Self {
        match value {
            0 => Move::U1,
            1 => Move::U2,
            2 => Move::U3,
            3 => Move::R1,
            4 => Move::R2,
            5 => Move::R3,
            6 => Move::F1,
            7 => Move::F2,
            8 => Move::F3,
            9 => Move::D1,
            10 => Move::D2,
            11 => Move::D3,
            12 => Move::L1,
            13 => Move::L2,
            14 => Move::L3,
            15 => Move::B1,
            16 => Move::B2,
            17 => Move::B3,
            _ => panic!(), //critical!("Invalid move index : {}", value),
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Move::U1 => "U1",
            Move::U2 => "U2",
            Move::U3 => "U3",
            Move::R1 => "R1",
            Move::R2 => "R2",
            Move::R3 => "R3",
            Move::F1 => "F1",
            Move::F2 => "F2",
            Move::F3 => "F3",
            Move::D1 => "D1",
            Move::D2 => "D2",
            Move::D3 => "D3",
            Move::L1 => "L1",
            Move::L2 => "L2",
            Move::L3 => "L3",
            Move::B1 => "B1",
            Move::B2 => "B2",
            Move::B3 => "B3",
            Move::Default => "__",
        };
        write!(f, "{}", name)
    }
}

/// Represents the basic symmetries of the cube. All 48 cube symmetries can be generated by sequences of these 4 symmetries.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BasicSymmetry {
    /// Rotate the URF corner cubie 120 degrees clockwise.
    RotUrf3 = 0,
    /// Rotate the face F2 180 degrees clockwise.
    RotF2 = 1,
    /// Rotate the whole cube around the U face 90 degrees clockwise.
    RotU4 = 2,
    /// Mirror the LR slice.
    MirrLr2 = 3,
}
