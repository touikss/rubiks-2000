use rand::Rng;

use super::{
    color::Color,
    image::Image,
    utils::*,
};

pub struct Shape {
    pixels: Vec<(u32, u32)>,
}

impl Shape {
    pub fn new() -> Self {
        Self {
            pixels: Vec::<(u32, u32)>::new(),
        }
    }

    pub fn add(&mut self, px: (u32, u32)) {
        self.pixels.push(px);
    }

    pub fn size(&self) -> usize {
        self.pixels.len()
    }

    pub fn draw_on(&self, image: &mut Image) {
        let mut rng = rand::thread_rng();

        let color = Color::new(
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
        );
        for (x, y) in &self.pixels {
            (*image).set_pixel_at(*x * RES_FACTOR, *y * RES_FACTOR, color);
        }
    }

    pub fn is_square(&self) -> bool {
        false
    }
}
