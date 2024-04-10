use std::fmt;

use super::utils::{self, *};

#[derive(Clone, Copy, Debug)]
pub enum ColorType {
    Red,
    Blue,
    Green,
    Yellow,
    Black,
    White,
    Orange,
    Empty,
}

impl ColorType {
    pub fn color(&self) -> Color {
        match self {
            ColorType::Red => RED,
            ColorType::Blue => BLUE,
            ColorType::Green => GREEN,
            ColorType::Yellow => YELLOW,
            ColorType::Black => BLACK,
            ColorType::White => WHITE,
            ColorType::Orange => ORANGE,
            ColorType::Empty => BLACK,
        }
    }

    pub fn iter() -> impl Iterator<Item = ColorType> {
        static COLORS: [ColorType; 7] = [
            ColorType::Red,
            ColorType::Blue,
            ColorType::Green,
            ColorType::Yellow,
            ColorType::Black,
            ColorType::White,
            ColorType::Orange,
        ];
        COLORS.iter().copied()
    }
}

#[derive(Clone, Copy)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn r(self) -> u8 {
        self.r
    }

    pub fn g(self) -> u8 {
        self.g
    }

    pub fn b(self) -> u8 {
        self.b
    }

    pub fn gray_scale(self) -> Self {
        let gray = utils::round(((self.r as u32 + self.g as u32 + self.b as u32) / 3) as u8, GRAY_SCALE_ROUND);
        
        Color::new(gray , gray , gray)
    }

    pub fn distance(self, other: Color) -> f64 {
        let square_diff = |a, b| (a as f64 - b as f64).powi(2);

        let sum_of_squared_diffs = square_diff(self.r(), other.r())
            + square_diff(self.g(), other.g())
            + square_diff(self.b(), other.b());

        sum_of_squared_diffs.sqrt()
    }

    pub fn estimate_color(self) -> ColorType {
        let mut min_dist = 100.;
        let mut res = ColorType::Empty;

        for color in ColorType::iter() {
            let curr_dist = self.distance(color.color());
            if curr_dist < min_dist {
                min_dist = curr_dist;
                res = color;
            }
        }
        res
    }

    pub fn round(&mut self, f: f32) {
        self.r = round(self.r, f);
        self.g = round(self.g, f);
        self.b = round(self.b, f);
    } 
}

impl fmt::Display for ColorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
