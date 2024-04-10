use rand::Rng;

use super::{
    color::Color,
    image::Image,
    utils::{BLUE, RES_FACTOR},
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
        // 1: min x
        // 2: max x
        // 3: min y
        // 4: max y

        let (x1, y1) = self.pixels.iter().min_by_key(|&&(x, _)| x).unwrap();
        let (x2, y2) = self.pixels.iter().max_by_key(|&&(x, _)| x).unwrap();
        let (x3, y3) = self.pixels.iter().min_by_key(|&&(_, y)| y).unwrap();
        let (x4, y4) = self.pixels.iter().max_by_key(|&&(_, y)| y).unwrap();

        if (*y1 as i32 - *y2 as i32).abs() < 5 && (*x3 as i32 - *x4 as i32).abs() < 5 {
            return true;
        }

        false
    }
}
