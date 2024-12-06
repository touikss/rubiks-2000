use std::path::PathBuf;
use std::sync::Mutex;

use crate::cube::utils::{ load_table_i8, load_table_u16, load_table_u32, load_table_u8 };

use super::cubie_cube::CubieCube;
use super::enums::{ BasicSymmetry as BS, ColorTag as Cl, Corner as Co, Edge as Ed, Facelet as Fc };
use lazy_static::lazy_static; // Used to create ALL_MOVES (because we cannot initialize with a not const functions a const at runtime)

//////////////////////// Some constants about Rubiks' Cube ////////////////////////////////////////////////////////////////

/// Number of possible orientations for corners.
pub const N_CORNER_ORI: usize = 2187;

/// Number of elements in the G_2 set.
pub const CARD_G_2: usize = 1_082_656;

/// Number of possible permutations for 4 edges.
pub const EDGE_PERM_4: usize = 4_804;

/// Number of possible orientations for edges.
pub const N_EDGE_ORI: usize = 2048;

/// Maximum depth for phase 1.
pub const MAX_DEPTH_PHASE_1: usize = 8;

/// Maximum depth for phase 2.
pub const MAX_DEPTH_PHASE_2: usize = 4;

/// Number of colors on the cube.
pub const N_COLORS: usize = 6;

/// Number of edges on the cube.
pub const N_EDGES: usize = 12;

/// Number of corners in the enum.
pub const N_CORNERS_ENUM: usize = 8;

/// Number of possible moves.
pub const N_MOVES: usize = 18;

/// Number of base moves.
pub const N_BASE_MOVES: usize = 6;

/// Number of permutations for 4 elements.
pub const N_PERM_4: usize = 24;

/// Number of combinations of choosing 4 from 8.
pub const N_CHOOSE_8_4: usize = 70;

/// Number of possible moves.
pub const N_MOVE: usize = 18;

/// Number of possible twist orientations.
pub const N_TWIST: usize = 2187;

/// Number of possible flip orientations.
pub const N_FLIP: usize = 2048;

/// Number of possible sorted slice positions.
pub const N_SLICE_SORTED: usize = 11880;

/// Number of possible slice positions.
pub const N_SLICE: usize = N_SLICE_SORTED / N_PERM_4;

/// Number of flip+slice classes for D4h symmetry group.
pub const N_FLIPSLICE_CLASS: usize = 64430;

/// Number of different positions of UR, UF, UL, and UB edges in phase 2.
pub const N_U_EDGES_PHASE2: usize = 1680;

/// Number of classes of equivalence for the D4h symmetry group.
pub const N_CORNERS_CLASS: usize = 2768;

/// Number of permutations for edges in U and D faces in phase 2.
pub const N_UD_EDGES: usize = 40320;

/// Number of cube symmetries in the full Oh group.
pub const N_SYM: usize = 48;

/// Number of symmetries in the D4h subgroup.
pub const N_SYM_D4H: usize = 16;

// Folder name for generated tables.
//pub const FOLDER: &str = "/";
lazy_static! {
    pub static ref FOLDER: PathBuf = {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tables");
        path
    };
}

/// Indicates if the cube is in a valid state.
pub const CUBE_OK: bool = true;

/// Number of possible corner permutations.
pub const N_CORNERS: usize = 40320;

/// Indicates a solved state.
pub const SOLVED: usize = 0;

/// Invalid value marker.
pub const INVALID: u16 = 65535;

/////////////////////// The default permutations and orientations of a cube ///////////////////////////////////////////////

// Corner permutations
pub const CP_DEFAULT: [Co; 8] = [
    Co::URF, //  URF
    Co::UFL, //  UFL
    Co::ULB, //  ULB
    Co::UBR, //  UBR
    Co::DFR, //  DFR
    Co::DLF, //  DLF
    Co::DBL, //  DBL
    Co::DRB, //  DRB
];

// Corner orientations
pub const CO_DEFAULT: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

// Edge permutations
pub const EP_DEFAULT: [Ed; 12] = [
    Ed::UR, // UR
    Ed::UF, // UF
    Ed::UL, // UL
    Ed::UB, // UB
    Ed::DR, // DR
    Ed::DF, // DF
    Ed::DL, // DL
    Ed::DB, // DB
    Ed::FR, // FR
    Ed::FL, // FL
    Ed::BL, // BL
    Ed::BR, // BR
];

// Edge orientation
pub const EO_DEFAULT: [u8; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

////////////////////// The permutations and orientations to create the basic moves ////////////////////////////////////////

pub const CP_U: [Co; 8] = [
    Co::UBR, //  URF
    Co::URF, //  UFL
    Co::UFL, //  ULB
    Co::ULB, //  UBR
    Co::DFR, //  DFR
    Co::DLF, //  DLF
    Co::DBL, //  DBL
    Co::DRB, //  DRB
];
pub const CO_U: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
pub const EP_U: [Ed; 12] = [
    Ed::UB, // UR
    Ed::UR, // UF
    Ed::UF, // UL
    Ed::UL, // UB
    Ed::DR, // DR
    Ed::DF, // DF
    Ed::DL, // DL
    Ed::DB, // DB
    Ed::FR, // FR
    Ed::FL, // FL
    Ed::BL, // BL
    Ed::BR, // BR
];
pub const EO_U: [u8; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub const CP_R: [Co; 8] = [
    Co::DFR, //  URF
    Co::UFL, //  UFL
    Co::ULB, //  ULB
    Co::URF, //  UBR
    Co::DRB, //  DFR
    Co::DLF, //  DLF
    Co::DBL, //  DBL
    Co::UBR, //  DRB
];
pub const CO_R: [u8; 8] = [2, 0, 0, 1, 1, 0, 0, 2];
pub const EP_R: [Ed; 12] = [
    Ed::FR, // UR
    Ed::UF, // UF
    Ed::UL, // UL
    Ed::UB, // UB
    Ed::BR, // DR
    Ed::DF, // DF
    Ed::DL, // DL
    Ed::DB, // DB
    Ed::DR, // FR
    Ed::FL, // FL
    Ed::BL, // BL
    Ed::UR, // BR
];
pub const EO_R: [u8; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub const CP_F: [Co; 8] = [
    Co::UFL, //  URF
    Co::DLF, //  UFL
    Co::ULB, //  ULB
    Co::UBR, //  UBR
    Co::URF, //  DFR
    Co::DFR, //  DLF
    Co::DBL, //  DBL
    Co::DRB, //  DRB
];
pub const CO_F: [u8; 8] = [1, 2, 0, 0, 2, 1, 0, 0];
pub const EP_F: [Ed; 12] = [
    Ed::UR, // UR
    Ed::FL, // UF
    Ed::UL, // UL
    Ed::UB, // UB
    Ed::DR, // DR
    Ed::FR, // DF
    Ed::DL, // DL
    Ed::DB, // DB
    Ed::UF, // FR
    Ed::DF, // FL
    Ed::BL, // BL
    Ed::BR, // BR
];
pub const EO_F: [u8; 12] = [0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0];

pub const CP_D: [Co; 8] = [
    Co::URF, //  URF
    Co::UFL, //  UFL
    Co::ULB, //  ULB
    Co::UBR, //  UBR
    Co::DLF, //  DFR
    Co::DBL, //  DLF
    Co::DRB, //  DBL
    Co::DFR, //  DRB
];
pub const CO_D: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
pub const EP_D: [Ed; 12] = [
    Ed::UR, // UR
    Ed::UF, // UF
    Ed::UL, // UL
    Ed::UB, // UB
    Ed::DF, // DR
    Ed::DL, // DF
    Ed::DB, // DL
    Ed::DR, // DB
    Ed::FR, // FR
    Ed::FL, // FL
    Ed::BL, // BL
    Ed::BR, // BR
];
pub const EO_D: [u8; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub const CP_L: [Co; 8] = [
    Co::URF, //  URF
    Co::ULB, //  UFL
    Co::DBL, //  ULB
    Co::UBR, //  UBR
    Co::DFR, //  DFR
    Co::UFL, //  DLF
    Co::DLF, //  DBL
    Co::DRB, //  DRB
];
pub const CO_L: [u8; 8] = [0, 1, 2, 0, 0, 2, 1, 0];
pub const EP_L: [Ed; 12] = [
    Ed::UR, // UR
    Ed::UF, // UF
    Ed::BL, // UL
    Ed::UB, // UB
    Ed::DR, // DR
    Ed::DF, // DF
    Ed::FL, // DL
    Ed::DB, // DB
    Ed::FR, // FR
    Ed::UL, // FL
    Ed::DL, // BL
    Ed::BR, // BR
];
pub const EO_L: [u8; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub const CP_B: [Co; 8] = [
    Co::URF, //  URF
    Co::UFL, //  UFL
    Co::UBR, //  ULB
    Co::DRB, //  UBR
    Co::DFR, //  DFR
    Co::DLF, //  DLF
    Co::ULB, //  DBL
    Co::DBL, //  DRB
];
pub const CO_B: [u8; 8] = [0, 0, 1, 2, 0, 0, 2, 1];
pub const EP_B: [Ed; 12] = [
    Ed::UR, // UR
    Ed::UF, // UF
    Ed::UL, // UL
    Ed::BR, // UB
    Ed::DR, // DR
    Ed::DF, // DF
    Ed::DL, // DL
    Ed::BL, // DB
    Ed::FR, // FR
    Ed::FL, // FL
    Ed::UB, // BL
    Ed::DB, // BR
];

pub const EO_B: [u8; 12] = [0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1];

////////////////////// The basic move represented with cubes ///////////////////////////////////////////////////////////////

// Identity move = default cube
pub const DEFAULT: CubieCube = CubieCube::new(CP_DEFAULT, CO_DEFAULT, EP_DEFAULT, EO_DEFAULT);

// Up move
pub const MOVE_U: CubieCube = CubieCube::new(CP_U, CO_U, EP_U, EO_U);

// Right move
pub const MOVE_R: CubieCube = CubieCube::new(CP_R, CO_R, EP_R, EO_R);

// Front move
pub const MOVE_F: CubieCube = CubieCube::new(CP_F, CO_F, EP_F, EO_F);

// Down move
pub const MOVE_D: CubieCube = CubieCube::new(CP_D, CO_D, EP_D, EO_D);

// Left move
pub const MOVE_L: CubieCube = CubieCube::new(CP_L, CO_L, EP_L, EO_L);

// Back move
pub const MOVE_B: CubieCube = CubieCube::new(CP_B, CO_B, EP_B, EO_B);

// The six basic moves
pub const BASE_MOVES: [CubieCube; 6] = [MOVE_U, MOVE_R, MOVE_F, MOVE_D, MOVE_L, MOVE_B];

// Initialize the ALL_MOVES static value
fn initialize_all_moves() -> [CubieCube; 18] {
    let mut all_move_tab = [DEFAULT; 18];

    for i in 0..N_BASE_MOVES {
        let mut cube = DEFAULT;
        for j in 0..3 {
            cube.multiply(BASE_MOVES[i]);
            all_move_tab[3 * i + j] = cube;
        }
    }

    all_move_tab
}

// Use lazy_static macros to initialize ALL_MOVES
// ALL_MOVES is (if we did it well) not modifiable
// ALL_MOVES contains all the six basic moves 1,2 and 3 times (thus len == 18)
lazy_static! {
    pub static ref ALL_MOVES: [CubieCube; 18] = initialize_all_moves();
}

//Map the corner positions to facelet positions.
// used to convert cube to face cube and vice versa
pub const CORNER_FACELET: [[Fc; 3]; 8] = [
    [Fc::U9, Fc::R1, Fc::F3], //  URF
    [Fc::U7, Fc::F1, Fc::L3], //  UFL
    [Fc::U1, Fc::L1, Fc::B3], //  ULB
    [Fc::U3, Fc::B1, Fc::R3], //  UBR
    [Fc::D3, Fc::F9, Fc::R7], //  DFR
    [Fc::D1, Fc::L9, Fc::F7], //  DLF
    [Fc::D7, Fc::B9, Fc::L7], //  DBL
    [Fc::D9, Fc::R9, Fc::B7], //  DRB
];

// Map the edge positions to facelet positions.
// used to convert cube to face cube and vice versa
pub const EDGE_FACELET: [[Fc; 2]; 12] = [
    [Fc::U6, Fc::R2], //  UR
    [Fc::U8, Fc::F2], //  UF
    [Fc::U4, Fc::L2], //  UL
    [Fc::U2, Fc::B2], //  UB
    [Fc::D6, Fc::R8], //  DR
    [Fc::D2, Fc::F8], //  DF
    [Fc::D4, Fc::L8], //  DL
    [Fc::D8, Fc::B8], //  DB
    [Fc::F6, Fc::R4], //  FR
    [Fc::F4, Fc::L6], //  FL
    [Fc::B6, Fc::L4], //  BL
    [Fc::B4, Fc::R6], //  BR
];

// Map the corner positions to facelet colors.
pub const CORNER_COLOR: [[Cl; 3]; 8] = [
    [Cl::U, Cl::R, Cl::F], //  URF
    [Cl::U, Cl::F, Cl::L], //  UFL
    [Cl::U, Cl::L, Cl::B], //  ULB
    [Cl::U, Cl::B, Cl::R], //  UBR
    [Cl::D, Cl::F, Cl::R], //  DFR
    [Cl::D, Cl::L, Cl::F], //  DLF
    [Cl::D, Cl::B, Cl::L], //  DBL
    [Cl::D, Cl::R, Cl::B], //  DRB
];

// Map the edge positions to facelet colors.
pub const EDGE_COLOR: [[Cl; 2]; 12] = [
    [Cl::U, Cl::R], //  UR
    [Cl::U, Cl::F], //  UF
    [Cl::U, Cl::L], //  UL
    [Cl::U, Cl::B], //  UB
    [Cl::D, Cl::R], //  DR
    [Cl::D, Cl::F], //  DF
    [Cl::D, Cl::L], //  DL
    [Cl::D, Cl::B], //  DB
    [Cl::F, Cl::R], //  FR
    [Cl::F, Cl::L], //  FL
    [Cl::B, Cl::L], //  BL
    [Cl::B, Cl::R], //  BR
];

lazy_static! {
    pub static ref U_EDGES_PLUS_D_EDGES_TO_UD_EDGES: Vec<u16> = load_table_u16(
        "phase2_edgemerge",
        N_U_EDGES_PHASE2 * N_PERM_4
    );
}

// ---------- MOVES -------------

lazy_static! {
    pub static ref TWIST_MOVE: Vec<u16> = load_table_u16("move_twist", N_TWIST * N_MOVE);
    pub static ref FLIP_MOVE: Vec<u16> = load_table_u16("move_flip", N_FLIP * N_MOVE);
    pub static ref SLICE_SORTED_MOVE: Vec<u16> = load_table_u16(
        "move_slice_sorted",
        N_SLICE_SORTED * N_MOVE
    );
    pub static ref U_EDGES_MOVE: Vec<u16> = load_table_u16("move_u_edges", N_SLICE_SORTED * N_MOVE);
    pub static ref D_EDGES_MOVE: Vec<u16> = load_table_u16("move_d_edges", N_SLICE_SORTED * N_MOVE);
    pub static ref UD_EDGES_MOVE: Vec<u16> = load_table_u16("move_ud_edges", N_UD_EDGES * N_MOVE);
    pub static ref CORNERS_MOVE: Vec<u16> = load_table_u16("move_corners", N_CORNERS * N_MOVE);
}

// -------------- PRUNNING -----------------
/// Creates a distance table.
pub fn create_distance_table() -> Vec<i8> {
    let size = 60;

    let mut dist = vec![0i8; size];

    for i in 0..20 {
        for j in 0..3 {
            dist[3 * i + j] = ((i / 3) * 3 + j) as i8;
            if i % 3 == 2 && j == 0 {
                dist[3 * i + j] += 3;
            } else if i % 3 == 0 && j == 2 {
                dist[3 * i + j] -= 3;
            }
        }
    }

    dist
}

lazy_static! {
    pub static ref FLIPSLICE_TWIST_DEPTH3: Mutex<Vec<u32>> = Mutex::new(
        load_table_u32("phase1_prun", (N_FLIPSLICE_CLASS * N_TWIST) / 16 + 1)
    );
    pub static ref CORNERS_UD_EDGES_DEPTH3: Mutex<Vec<u32>> = Mutex::new(
        load_table_u32("phase2_prun", (N_CORNERS_CLASS * N_UD_EDGES) / 16)
    );
    pub static ref CORNSLICE_DEPTH: Mutex<Vec<i8>> = Mutex::new(
        load_table_i8("phase2_cornsliceprun", N_CORNERS * N_PERM_4)
    );
    pub static ref DISTANCE: Vec<i8> = create_distance_table();
}

// ----------- SYMMETRIES --------------
// 120° clockwise rotation around the long diagonal URF-DBL
const CP_ROT_URF3: [Co; 8] = [
    Co::URF,
    Co::DFR,
    Co::DLF,
    Co::UFL,
    Co::UBR,
    Co::DRB,
    Co::DBL,
    Co::ULB,
];
const CO_ROT_URF3: [u8; 8] = [1, 2, 1, 2, 2, 1, 2, 1];
const EP_ROT_URF3: [Ed; 12] = [
    Ed::UF,
    Ed::FR,
    Ed::DF,
    Ed::FL,
    Ed::UB,
    Ed::BR,
    Ed::DB,
    Ed::BL,
    Ed::UR,
    Ed::DR,
    Ed::DL,
    Ed::UL,
];
const EO_ROT_URF3: [u8; 12] = [1, 0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1];

// 180° rotation around the axis through the F and B centers
const CP_ROT_F2: [Co; 8] = [Co::DLF, Co::DFR, Co::DRB, Co::DBL, Co::UFL, Co::URF, Co::UBR, Co::ULB];
const CO_ROT_F2: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
const EP_ROT_F2: [Ed; 12] = [
    Ed::DL,
    Ed::DF,
    Ed::DR,
    Ed::DB,
    Ed::UL,
    Ed::UF,
    Ed::UR,
    Ed::UB,
    Ed::FL,
    Ed::FR,
    Ed::BR,
    Ed::BL,
];
const EO_ROT_F2: [u8; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

// 90° clockwise rotation around the axis through the U and D centers
const CP_ROT_U4: [Co; 8] = [Co::UBR, Co::URF, Co::UFL, Co::ULB, Co::DRB, Co::DFR, Co::DLF, Co::DBL];
const CO_ROT_U4: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
const EP_ROT_U4: [Ed; 12] = [
    Ed::UB,
    Ed::UR,
    Ed::UF,
    Ed::UL,
    Ed::DB,
    Ed::DR,
    Ed::DF,
    Ed::DL,
    Ed::BR,
    Ed::FR,
    Ed::FL,
    Ed::BL,
];
const EO_ROT_U4: [u8; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1];

// Reflection at the plane through the U, D, F, B centers
const CP_MIRR_LR2: [Co; 8] = [
    Co::UFL,
    Co::URF,
    Co::UBR,
    Co::ULB,
    Co::DLF,
    Co::DFR,
    Co::DRB,
    Co::DBL,
];
const CO_MIRR_LR2: [u8; 8] = [3, 3, 3, 3, 3, 3, 3, 3];
const EP_MIRR_LR2: [Ed; 12] = [
    Ed::UL,
    Ed::UF,
    Ed::UR,
    Ed::UB,
    Ed::DL,
    Ed::DF,
    Ed::DR,
    Ed::DB,
    Ed::FL,
    Ed::FR,
    Ed::BR,
    Ed::BL,
];
const EO_MIRR_LR2: [u8; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

lazy_static! {
    pub static ref BASIC_SYM_CUBE: [CubieCube; 4] = {
        let mut arr = [DEFAULT; 4];
        arr[BS::RotUrf3 as usize] = CubieCube::new(
            CP_ROT_URF3,
            CO_ROT_URF3,
            EP_ROT_URF3,
            EO_ROT_URF3
        );
        arr[BS::RotF2 as usize] = CubieCube::new(CP_ROT_F2, CO_ROT_F2, EP_ROT_F2, EO_ROT_F2);
        arr[BS::RotU4 as usize] = CubieCube::new(CP_ROT_U4, CO_ROT_U4, EP_ROT_U4, EO_ROT_U4);
        arr[BS::MirrLr2 as usize] = CubieCube::new(
            CP_MIRR_LR2,
            CO_MIRR_LR2,
            EP_MIRR_LR2,
            EO_MIRR_LR2
        );
        arr
    };
}

// Définir une fonction pour créer les instances de symCube
fn create_sym_cube() -> [CubieCube; N_SYM] {
    let mut sym_c = [DEFAULT; N_SYM];
    let mut cc = DEFAULT;
    let mut idx = 0;
    for _urf3 in 0..3 {
        for _f2 in 0..2 {
            for _u4 in 0..4 {
                for _d_edges_movelr2 in 0..2 {
                    sym_c[idx] = cc.clone();
                    idx += 1;
                    cc.multiply(BASIC_SYM_CUBE[BS::MirrLr2 as usize]);
                }
                cc.multiply(BASIC_SYM_CUBE[BS::RotU4 as usize]);
            }
            cc.multiply(BASIC_SYM_CUBE[BS::RotF2 as usize]);
        }
        cc.multiply(BASIC_SYM_CUBE[BS::RotUrf3 as usize]);
    }
    sym_c
}

// Définir une fonction pour créer les indices d'inversion
fn create_inv_idx(sym_c: &[CubieCube; N_SYM]) -> [u8; N_SYM] {
    let mut invv_idx: [u8; N_SYM] = [0; N_SYM];
    for j in 0..N_SYM {
        for i in 0..N_SYM {
            let mut temp_cc = sym_c[j].clone();
            temp_cc.corner_multiply(sym_c[i]);
            if
                temp_cc.cp[Co::URF as usize] == Co::URF &&
                temp_cc.cp[Co::UFL as usize] == Co::UFL &&
                temp_cc.cp[Co::ULB as usize] == Co::ULB
            {
                invv_idx[j] = i as u8;
                break;
            }
        }
    }
    invv_idx
}

// Définir une fonction pour créer l'array mult_sym
fn create_mult_sym(sym_c: &[CubieCube; N_SYM]) -> [u8; N_SYM * N_SYM] {
    let mut mul_sym: [u8; N_SYM * N_SYM] = [0; N_SYM * N_SYM];
    for i in 0..N_SYM {
        for j in 0..N_SYM {
            let mut cc = sym_c[i].clone();
            cc.multiply(sym_c[j]);
            for k in 0..N_SYM {
                if cc == sym_c[k] {
                    mul_sym[N_SYM * i + j] = k as u8;
                    break;
                }
            }
        }
    }
    mul_sym
}

// Définir une fonction pour créer l'array conj_move
fn create_conj_move(sym_c: &[CubieCube; N_SYM], invv_idx: &[u8; N_SYM]) -> [u16; N_MOVE * N_SYM] {
    let mut conjj_move: [u16; N_MOVE * N_SYM] = [0; N_MOVE * N_SYM];
    for s in 0..N_SYM {
        for m in 0..N_MOVE {
            let mut ss = sym_c[s].clone();
            ss.multiply(ALL_MOVES[m]);
            ss.multiply(sym_c[invv_idx[s] as usize]);
            for m2 in 0..N_MOVE {
                if ss == ALL_MOVES[m2] {
                    conjj_move[N_MOVE * s + m] = m2 as u16;
                    break;
                }
            }
        }
    }
    conjj_move
}

lazy_static! {
    pub static ref SYM_CUBE: [CubieCube; N_SYM] = create_sym_cube();
    pub static ref INV_IDX: [u8; N_SYM] = create_inv_idx(&SYM_CUBE);
    pub static ref MULT_SYM: [u8; N_SYM * N_SYM] = create_mult_sym(&SYM_CUBE);
    pub static ref CONJ_MOVE: [u16; N_MOVE * N_SYM] = create_conj_move(&SYM_CUBE, &INV_IDX);
    pub static ref TWIST_CONJ: Vec<u16> = load_table_u16("conj_twist", N_TWIST * N_SYM_D4H);
    pub static ref UD_EDGES_CONJ: Vec<u16> = load_table_u16(
        "conj_ud_edges",
        N_UD_EDGES * N_SYM_D4H
    );
    pub static ref FLIPSLICE_CLASSIDX: Vec<u16> = load_table_u16("fs_classidx", N_FLIP * N_SLICE);
    pub static ref FLIPSLICE_SYM: Vec<u8> = load_table_u8("fs_sym", N_FLIP * N_SLICE);
    pub static ref FLIPSLICE_REP: Vec<u32> = load_table_u32("fs_rep", N_FLIPSLICE_CLASS);
    pub static ref CORNER_CLASSIDX: Vec<u16> = load_table_u16("co_classidx", N_CORNERS);
    pub static ref CORNER_SYM: Vec<u8> = load_table_u8("co_sym", N_CORNERS);
    pub static ref CORNER_REP: Vec<u16> = load_table_u16("co_rep", N_CORNERS_CLASS);
}
