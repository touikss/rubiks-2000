use crate::imagery::{
    color, shape,
    utils::{round, GREEN, ORANGE, RED, WHITE, YELLOW},
};

use super::{color::Color, shape::*, utils::*};

use gtk::{ffi::GTK_CALENDAR_NO_MONTH_CHANGE, gdk::shows_events};
use rand::Rng;

use std::collections::HashMap;

#[derive(Clone)]
pub struct Image {
    bytes: Vec<u8>,
    width: u32,
    height: u32,
}

impl Image {
    // Create a new image from a frame given by rscam
    pub fn new_frome_frame(frame: &rscam::Frame) -> Self {
        Image {
            bytes: frame.to_vec(),
            width: frame.resolution.0,
            height: frame.resolution.1,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn bytes(&self) -> &Vec<u8> {
        &self.bytes
    }

    // Return the color of a pixel at a given position
    pub fn pixel_at(&self, x: u32, y: u32) -> Option<Color> {
        if x < self.width && y < self.height {
            let index = ((y * self.width + x) * 3) as usize;
            Some(Color::new(
                self.bytes[index],
                self.bytes[index + 1],
                self.bytes[index + 2],
            ))
        } else {
            None
        }
    }

    // Return the color of a pixel at a given position
    pub fn set_pixel_at(&mut self, x: u32, y: u32, color: Color) {
        for px in x..x + RES_FACTOR {
            for py in y..y + RES_FACTOR {
                if px < self.width && py < self.height {
                    let index = ((py * self.width + px) * 3) as usize;
                    self.bytes[index] = color.r();
                    self.bytes[index + 1] = color.g();
                    self.bytes[index + 2] = color.b();
                }
            }
        }
    }

    // Apply a gray filter to the image
    pub fn apply_gray_filter(&mut self) {
        for x in (0..self.width).step_by(RES_FACTOR as usize) {
            for y in (0..self.height).step_by(RES_FACTOR as usize) {
                self.set_pixel_at(x, y, self.pixel_at(x, y).unwrap().gray_scale());
            }
        }
    }

    // Apply a Gaussian blur n*n
    pub fn apply_blur(&mut self, n: u32) {
        let mut blurred_image = self.clone();

        for x in 0..self.width {
            for y in 0..self.height {
                let mut sum_red = 0;
                let mut sum_green = 0;
                let mut sum_blue = 0;

                for i in (x.saturating_sub(n)..=x.saturating_add(n)).filter(|&i| i < self.width) {
                    for j in
                        (y.saturating_sub(n)..=y.saturating_add(n)).filter(|&j| j < self.height)
                    {
                        if let Some(color) = self.pixel_at(i, j) {
                            sum_red += color.r() as u32;
                            sum_green += color.g() as u32;
                            sum_blue += color.b() as u32;
                        }
                    }
                }

                let count = (2 * n + 1).pow(2) as u32;
                let avg_red = (sum_red / count) as u8;
                let avg_green = (sum_green / count) as u8;
                let avg_blue = (sum_blue / count) as u8;

                blurred_image.set_pixel_at(x, y, Color::new(avg_red, avg_green, avg_blue));
            }
        }
        *self = blurred_image;
    }

    pub fn apply_treshold(&mut self, threshold: u8) {
      
        for x in 0..self.width {
            for y in 0..self.height {
              
                if let Some(color) = self.pixel_at(x, y) {
                    
                    let luminance = (color.r() as f32 * 0.299
                        + color.g() as f32 * 0.587
                        + color.b() as f32 * 0.114) as u8;
                  
                    if luminance > threshold {
                       
                        self.set_pixel_at(x, y, Color::new(255, 255, 255));
                    } else {
                        
                        self.set_pixel_at(x, y, Color::new(0, 0, 0));
                    }
                }
            }
        }
    }

    // Apply the mass detection
    pub fn apply_mass_detection(&mut self) {
        let mut rng = rand::thread_rng();

        let mut count: u32 = 0;

        let n_width = self.width / RES_FACTOR;
        let n_height = self.height / RES_FACTOR;

        let mut marqued: Vec<Vec<bool>> = vec![vec![false; n_height as usize]; n_width as usize];

        let mut masses: Vec<Shape> = vec![];

        for px in 0..n_width {
            for py in 0..n_height {
                if marqued[px as usize][py as usize] {
                    continue;
                }

                count += 1;

                let mut pix_queue = vec![];
                let mut shape = Shape::new();
                pix_queue.push((px as i32, py as i32));
                shape.add((px, py));

                marqued[px as usize][py as usize] = true;
                let mass_color = self.pixel_at(px * RES_FACTOR, py * RES_FACTOR).unwrap();

                while pix_queue.len() > 0 {
                    let (x, y) = pix_queue.pop().unwrap();

                    for i in x - 1..x + 2 {
                        if i < 0 || i >= n_width as i32 {
                            continue;
                        }

                        for j in y - 1..y + 2 {
                            if j < 0 || j >= n_height as i32 || marqued[i as usize][j as usize] {
                                continue;
                            }

                            if let Some(mut color) =
                                self.pixel_at((i as u32) * RES_FACTOR, (j as u32) * RES_FACTOR)
                            {
                                color.round(10.);
                                if color.distance(mass_color) < DIST_MASS_DETECTION {
                                    pix_queue.push((i, j));
                                    shape.add((i as u32, j as u32));
                                    marqued[i as usize][j as usize] = true;
                                }
                            }
                        }
                    }
                }

                if shape.size() >  10 && shape.size() <  150{
                    shape.draw_on(self);
                }
            }
        }
        dbg!(count);
    }

    pub fn apply_histogram(&mut self) {
        // Initialisation des histogrammes pour les canaux rouge, vert et bleu
        let mut red_histogram: HashMap<u8, usize> = HashMap::new();
        let mut green_histogram: HashMap<u8, usize> = HashMap::new();
        let mut blue_histogram: HashMap<u8, usize> = HashMap::new();

        // Parcours de chaque triplet de valeurs (rouge, vert, bleu) dans l'image
        for chunk in self.bytes.chunks(3) {
            // Récupération des valeurs de chaque canal
            let red = chunk[0];
            let green = chunk[1];
            let blue = chunk[2];

            // Mise à jour de l'histogramme pour chaque canal
            *red_histogram.entry(red).or_insert(0) += 1;
            *green_histogram.entry(green).or_insert(0) += 1;
            *blue_histogram.entry(blue).or_insert(0) += 1;
        }
    }

    pub fn equalize(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {}
        }
    }

    pub fn reduce(&mut self) {
        for x in (0..self.width).step_by(RES_FACTOR as usize) {
            for y in (0..self.height).step_by(RES_FACTOR as usize) {
                let color = self.pixel_at(x, y).unwrap();
                for cx in x..x + RES_FACTOR {
                    for cy in y..y + RES_FACTOR {
                        self.set_pixel_at(cx, cy, color);
                    }
                }
            }
        }
    }

    // Draw a line on the image
    pub fn draw_line(&mut self, x1: u32, y1: u32, x2: u32, y2: u32) {
        // Vérifier si les coordonnées sont dans les limites de l'image
        if x1 >= self.width || y1 >= self.height || x2 >= self.width || y2 >= self.height {
            panic!("Coordonnées en dehors des limites de l'self");
        }

        // Convertir les coordonnées (x, y) en index dans le vecteur image
        let index = |x: u32, y: u32| (y * self.width * 3) + (x * 3);

        // Déterminer les deltas pour x et y
        let dx = (x2 as isize) - (x1 as isize);
        let dy = (y2 as isize) - (y1 as isize);

        // Déterminer si la pente est raide ou peu raide
        let steep = dy.abs() > dx.abs();

        // Si la pente est raide, échanger x et y
        let (mut x, mut y, dx, dy) = if steep {
            (y1 as isize, x1 as isize, dy, dx)
        } else {
            (x1 as isize, y1 as isize, dx, dy)
        };

        // Déterminer l'incrémentation pour x et y
        let increment_x = if dx < 0 { -1 } else { 1 };
        let increment_y = if dy < 0 { -1 } else { 1 };

        // Convertir les deltas en valeurs absolues
        let dx = dx.abs();
        let dy = dy.abs();

        // Déterminer les étapes maximales
        let max_steps = if dx > dy { dx } else { dy };

        // Calculer l'erreur initiale
        let mut error = max_steps / 2;

        // Dessiner la ligne pixel par pixel
        for _ in 0..=max_steps {
            // Calculer l'index du pixel dans le vecteur image
            let pixel_index = if steep {
                index(y as u32, x as u32) as usize
            } else {
                index(x as u32, y as u32) as usize
            };

            // Dessiner le pixel de couleur verte (RGB: 0, 255, 0)
            self.bytes[pixel_index] = 0; // Rouge
            self.bytes[pixel_index + 1] = 255; // Vert
            self.bytes[pixel_index + 2] = 0; // Bleu

            // Mise à jour de l'erreur
            error -= dy;

            // Si l'erreur est négative, avancer dans la direction y et ajouter à l'erreur
            if error < 0 {
                y += increment_y;
                error += dx;
            }

            // Avancer dans la direction x
            x += increment_x;
        }
    }

    // Draw an empty square on the image
    pub fn draw_square(&mut self, x: u32, y: u32, length: u32) {
        self.draw_line(x, y, x, y + length);
        self.draw_line(x, y, x + length, y);
        self.draw_line(x + length, y, x + length, y + length);
        self.draw_line(x, y + length, x + length, y + length);
    }

    /*pub fn get_pixel_color(&self, x: u32, y: u32, length: u32) {
        // Vérifier si les coordonnées sont dans les limites de l'image
        if x >= *self.width() || y >= self.height() {
            panic!("Coordonnées en dehors des limites de l'self");
        }

        // Convertir les coordonnées (x, y) en index dans le vecteur self
        let red = |x: u32, y: u32| {
            let index = (y * self.width() * 3) + (x * 3) + 0;
            self.bytes[index as usize]
        };

        let green = |x: u32, y: u32| {
            let index = (y * self.width() * 3) + (x * 3) + 1;
            self.bytes[index as usize]
        };

        let blue = |x: u32, y: u32| {
            let index = (y * self.width() * 3) + (x * 3) + 2;
            self.bytes[index as usize]
        };

        let (mut r, mut g, mut b) = (0., 0., 0.);

        for i in x..x + length {
            for j in y..y + length {
                r += red(i, j) as f64;
                g += green(i, j) as f64;
                b += blue(i, j) as f64;
            }
        }

        r /= (length * length) as f64;
        g /= (length * length) as f64;
        b /= (length * length) as f64;

        let color = (r as u32, g as u32, b as u32);

        //println!("r : {}, g : {}, b : {}", r, g, b);

        //let mut color_dist = vec![];

        /*color_dist.push(color_distance(color, BLUE));
        color_dist.push(color_distance(color, YELLOW));
        color_dist.push(color_distance(color, RED));
        color_dist.push(color_distance(color, BLACK));
        color_dist.push(color_distance(color, ORANGE));
        color_dist.push(color_distance(color, GREEN));*/

        /*let (index, _min_value) = color_dist
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();

        let s = match index {
            0 => "BLUE",
            1 => "YELLOW",
            2 => "RED",
            3 => "BLACK",
            4 => "ORANGE",
            5 => "GREEN",
            _ => "NOP",
        };*/

        //dbg!(s);
    }*/
}

pub struct ImageIterator<'a> {
    image: &'a Image,
    x: u32,
    y: u32,
}

impl<'a> ImageIterator<'a> {
    pub fn new(image: &'a Image) -> Self {
        Self { image, x: 0, y: 0 }
    }
}

impl<'a> Iterator for ImageIterator<'a> {
    type Item = (u32, u32, Color);

    fn next(&mut self) -> Option<Self::Item> {
        if self.y >= self.image.height {
            return None;
        }

        let pixel = self.image.pixel_at(self.x, self.y);
        if self.x + 1 < self.image.width {
            self.x += 1;
        } else {
            self.x = 0;
            self.y += 1;
        }
        pixel.map(|p| (self.x, self.y, p))
    }
}

impl<'a> IntoIterator for &'a Image {
    type Item = (u32, u32, Color);
    type IntoIter = ImageIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ImageIterator::new(self)
    }
}
