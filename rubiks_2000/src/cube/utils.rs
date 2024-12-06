//! This module contains usefull functions for cube solver.

use std::{fs::File, io::Read};

use crate::{info, CORNERS_UD_EDGES_DEPTH3, FLIPSLICE_TWIST_DEPTH3, FOLDER};

/// Calculates the binomial coefficient "n choose k".
///
/// # Args
///
/// * `n`: The total number of elements.
/// * `k`: The number of elements to choose.
///
/// # Returns
///
/// The binomial coefficient "n choose k".
pub fn c_nk(n: usize, k: usize) -> usize {
    if n < k {
        return 0;
    }
    let mut k = k;
    if k > n / 2 {
        k = n - k;
    }
    let mut s = 1;
    let mut i = n;
    let mut j = 1;
    while i != n - k {
        s *= i;
        s /= j;
        i -= 1;
        j += 1;
    }
    s
}

/// Rotates the elements of a mutable slice to the right within the specified range.
///
/// # Arguments
///
/// * `arr` - A mutable reference to the slice to be rotated.
/// * `left` - The left index of the range to be rotated.
/// * `right` - The right index of the range to be rotated.
pub fn rotate_right<T: Clone>(arr: &mut [T], left: usize, right: usize) {
    let temp = arr[right].clone();
    for i in (left + 1..=right).rev() {
        arr[i] = arr[i - 1].clone();
    }
    arr[left] = temp;
}

/// Rotates the elements of a mutable slice to the left within the specified range.
///
/// # Arguments
///
/// * `arr` - A mutable reference to the slice to be rotated.
/// * `left` - The left index of the range to be rotated.
/// * `right` - The right index of the range to be rotated.
pub fn rotate_left<T: Clone>(arr: &mut [T], left: usize, right: usize) {
    let temp = arr[left].clone();
    for i in left..right {
        arr[i] = arr[i + 1].clone();
    }
    arr[right] = temp;
}

// ---------- PRUNNING ----------

/// Retrieves the flip slice twist depth 3 value at the specified index.
///
/// # Arguments
///
/// * `ix` - The index of the value to retrieve.
///
/// # Returns
///
/// The flip slice twist depth 3 value at the specified index.
pub fn get_flipslice_twist_depth3(ix: usize) -> u8 {
    let flip = FLIPSLICE_TWIST_DEPTH3.lock().unwrap();

    let base = ix / 16;
    let shift = (ix % 16) * 2;
    let value = (flip[base] >> shift) & 3;
    value as u8
}

/// Retrieves the corners UD edges depth 3 value at the specified index.
///
/// # Arguments
///
/// * `ix` - The index of the value to retrieve.
///
/// # Returns
///
/// The corners UD edges depth 3 value at the specified index.
pub fn get_corners_ud_edges_depth3(ix: usize) -> u32 {
    let mut y = CORNERS_UD_EDGES_DEPTH3.lock().unwrap()[(ix / 16) as usize];
    y >>= (ix % 16) * 2;
    y & 3
}

/// Sets the flip slice twist depth 3 value at the specified index.
///
/// # Arguments
///
/// * `ix` - The index where the value will be set.
/// * `value` - The value to set.
pub fn set_flipslice_twist_depth3(ix: usize, value: u8) {
    let mut flip = FLIPSLICE_TWIST_DEPTH3.lock().unwrap();

    let base = ix / 16;
    let shift = (ix % 16) * 2;
    let mask = !(3 << shift);
    flip[base] &= mask;
    flip[base] |= (value as u32) << shift;
}

/// Sets the corners UD edges depth 3 value at the specified index.
///
/// # Arguments
///
/// * `ix` - The index where the value will be set.
/// * `value` - The value to set.
pub fn set_corners_ud_edges_depth3(ix: usize, value: u8) {
    let mut corners = CORNERS_UD_EDGES_DEPTH3.lock().unwrap();

    let base = ix / 16;
    let shift = (ix % 16) * 2;
    let mask = !(3 << shift);
    corners[base] &= mask;
    corners[base] |= (value as u32) << shift;
}

// ---------- LOADING TABLES ----------

/// Loads a table of u16 values from a file.
///
/// # Arguments
///
/// * `fname` - The name of the file containing the table.
/// * `length` - The length of the table to load.
///
/// # Returns
///
/// A vector containing the loaded table.
pub fn load_table_u16(fname: &str, length: usize) -> Vec<u16> {
    let file_path = FOLDER.join(fname);
    info!(
        "loading {} table at {} ...",
        fname,
        file_path.to_str().unwrap()
    );
    let mut file = File::open(&file_path).expect("Failed to open file");
    let mut res: Vec<u16> = vec![0; length];
    file.read_exact(unsafe {
        std::slice::from_raw_parts_mut(res.as_mut_ptr() as *mut u8, res.len() * 2)
    })
    .expect("Failed to read file");
    res
}

/// Loads a table of u8 values from a file.
///
/// # Arguments
///
/// * `fname` - The name of the file containing the table.
/// * `length` - The length of the table to load.
///
/// # Returns
///
/// A vector containing the loaded table.
pub fn load_table_u8(fname: &str, length: usize) -> Vec<u8> {
    info!("Loading {} table...", fname);
    let file_path = FOLDER.join(fname);
    let mut file = File::open(&file_path).expect("Failed to open file");
    let mut data: Vec<u8> = vec![0; length]; // Remplacez N_FLIP * N_SLICE par la taille réelle de votre tableau
    file.read_exact(&mut data).expect("Failed to read file");
    data
}

/// Loads a table of i8 values from a file.
///
/// # Arguments
///
/// * `fname` - The name of the file containing the table.
/// * `length` - The length of the table to load.
///
/// # Returns
///
/// A vector containing the loaded table.
pub fn load_table_i8(fname: &str, length: usize) -> Vec<i8> {
    info!("Loading {} table...", fname);
    let file_path = FOLDER.join(fname);
    let mut file = File::open(&file_path).expect("Failed to open file");
    let mut data: Vec<i8> = vec![0; length]; // Remplacez N_FLIP * N_SLICE par la taille réelle de votre tableau
    file.read_exact(unsafe {
        std::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut u8, data.len())
    })
    .expect("Failed to read file");
    data
}

/// Loads a table of u32 values from a file.
///
/// # Arguments
///
/// * `fname` - The name of the file containing the table.
/// * `length` - The length of the table to load.
///
/// # Returns
///
/// A vector containing the loaded table.
pub fn load_table_u32(fname: &str, length: usize) -> Vec<u32> {
    info!("Loading {} table...", fname);
    let file_path = FOLDER.join(fname);
    let mut file = File::open(&file_path).expect("Failed to open file");
    let mut data: Vec<u32> = vec![0; length]; // Remplacez N_FLIPSLICE_CLASS par la taille réelle de votre tableau
    file.read_exact(unsafe {
        std::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut u8, data.len() * 4)
    })
    .expect("Failed to read file");
    data
}
