#![allow(dead_code)] // This macros disable all the "unsed function/variable ..." warning, to simplify clarity during dev

mod cube;
mod table;
use crate::cube::{defs::*, enums::Move, cube::Cube};

fn print_move_sequenve(move_sequence: &[Move]) {
    for mv in move_sequence {
        print!("{} ", mv);
    }
    println!();
}

// Test module
#[cfg(test)]
mod tests;

fn main() {
    // CUbe de base
    let mut c = Cube::new(CP_DEFAULT, CO_DEFAULT, EP_DEFAULT, EO_DEFAULT);
    
    // applique les moves
    c.multiply(MOVE_D);
    c.multiply(MOVE_B);

    // converti en string
    let s = c.to_string();

    dbg!(s);
}
