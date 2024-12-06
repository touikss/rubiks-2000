use super::color::{ColorTable, RgbColor};

// default colors
pub const BLUE: RgbColor = RgbColor::new(0, 0, 255);       
pub const YELLOW: RgbColor = RgbColor::new(255, 255, 0);    
pub const RED: RgbColor = RgbColor::new(255, 0, 0);         
pub const BLACK: RgbColor = RgbColor::new(0, 0, 0);         
pub const ORANGE: RgbColor = RgbColor::new(255, 165, 0);    
pub const GREEN: RgbColor = RgbColor::new(0, 255, 0);       
pub const WHITE: RgbColor = RgbColor::new(255, 255, 255);   
pub const GRAY: RgbColor = RgbColor::new(128, 128, 128);  

// consts for shape and shape tab
pub const MIN_X: usize = 0;
pub const MAX_X: usize = 1;
pub const MIN_Y: usize = 2;
pub const MAX_Y: usize = 3;

// position of the neighbors
pub const UP: usize = 0;
pub const RIGHT: usize = 1;
pub const DOWN: usize = 2;
pub const LEFT: usize = 3;

pub const DEFAULT_COLOR_DATA: ColorTable = ColorTable::new();


// detection consts
pub const GRAY_SCALE_ROUND: f32 = 5.;
pub const DIST_MASS_DETECTION: f64 = 30.;
pub const RES_FACTOR: u32 = 5;

pub const SHAPE_DELTA: i32 = 300;

// util functions

pub fn round(num: u8, r: f32) -> u8 {
    ((num as f32 / r).round() * r) as u8
}

pub fn calculate_mean(data: &[f64]) -> f64 {
    let sum: f64 = data.iter().sum();
    sum / data.len() as f64
}

pub fn calculate_std_dev(data: &[f64]) -> f64 {
    let mean = calculate_mean(data);
    let squared_diff_sum: f64 = data.iter().map(|x| (x - mean).powi(2)).sum();
    let variance = squared_diff_sum / data.len() as f64;
    variance.sqrt()
}
