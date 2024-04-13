use std::fmt;

use super::utils::{self, *};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColorType {
    Red,
    Orange,
    Blue,
    Yellow,
    Green,
    White,
    Black,
    Empty,
    Other
}

impl ColorType {
    pub fn to_rgb(&self) -> (f32, f32, f32) {
        let (r, g, b) = match self {
            ColorType::Red => (1.0, 0.0, 0.0),
            ColorType::Orange => (1.0, 0.5, 0.0),
            ColorType::Blue => (0., 0., 1.0),
            ColorType::Yellow => (1.0, 1.0, 0.0),
            ColorType::Green => (0., 1., 0.),
            ColorType::White => (1., 1., 1.),
            ColorType::Empty => (0., 0., 0.),
            ColorType::Other => (0., 0., 0.),
            ColorType::Black => (0., 0., 0.),
        };
        (r, g, b)
    }

    pub fn pretty_print(&self) {
        let code = match self {
            Self::Blue => "\x1b[34m",
            Self::White => "\x1b[37m",
            Self::Green => "\x1b[32m",
            Self::Red => "\x1b[31m",
            Self::Orange => "\x1b[38;5;208m",
            Self::Yellow => "\x1b[33m",
            _ => "\x1b[30m",
        };
        let reset_code = "\x1b[0m"; // Code pour réinitialiser la couleur à la couleur par défaut
        print!("{}██{}", code, reset_code);
    }

    pub fn color(&self) -> Color {
        match self {
            ColorType::Red => RED,
            ColorType::Blue => BLUE,
            ColorType::Green => GREEN,
            ColorType::Yellow => YELLOW,
            ColorType::Black => BLACK,
            ColorType::White => WHITE,
            ColorType::Orange => ORANGE,
            _ => BLACK,
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
