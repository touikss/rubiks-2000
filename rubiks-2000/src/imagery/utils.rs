use super::color::Color;

pub const BLUE: Color = Color::new(30, 80, 100);
pub const YELLOW: Color = Color::new(120, 130, 75);
pub const RED: Color = Color::new(110, 60, 40);
pub const BLACK: Color = Color::new(35, 50, 50);
pub const ORANGE: Color = Color::new(110, 80, 50);
pub const GREEN: Color = Color::new(60, 100, 60);
pub const WHITE: Color = Color::new(200, 200, 200);

pub const GRAY_SCALE_ROUND: f32 = 5.;
pub const DIST_MASS_DETECTION: f64 = 30.;
pub const RES_FACTOR: u32 = 5;

pub fn round(num: u8, r: f32) -> u8 {
    ((num as f32 / r).round() * r) as u8
}
