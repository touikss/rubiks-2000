//! This module contains the [`Shape`] definition for the square detection.

// std
use rand::Rng;

// local
use super::{ color::{ColorTable, RgbColor}, image::Image, utils::* };

/// A shape of an image represented by its pixel positions.
///
/// ## Fields
/// * `pixels` - the pixels of the shape (position, intensity)
/// * `center` - the center of the shape
/// * `extremums` - the extremums of the shape
/// * `width, height` - the dimentions of the shape
/// * `mark` - the mark color of the shape (not the real color on the image)
/// * `angle` - the inclination of the shape
#[derive(Clone, PartialEq)]
pub struct Shape {
    /// The pixels of the shape (position, intensity)
    pixels: Vec<(u32, u32, f32)>,
    /// The center of the shape
    pub(super) center: (u32, u32),
    /// The extremums of the shape
    extremums: Vec<(u32, u32)>,
    /// The width of the shape
    width: u32,
    /// The height of the shape
    height: u32,
    /// The mark color of the shape (not the real color on the image)
    mark: (u32, u32, u32),
    /// The inclination of the shape
    angle: f32,
    /// The neighbors of the shape (the indices in the [`ShapeTab`] containing it)
    pub(super) neighbors: [Option<usize>; 4],
}

impl Shape {
    /// Create a new shape.
    ///
    /// ## Args
    ///
    /// * `pixels` - the pixels of the shape
    ///
    /// ## Notes
    ///
    /// * `enter` is computed.
    /// * `extremums` are computed.
    /// * `width` and `height` are computed.
    /// * `angle` is computed
    pub fn new(pixels: Vec<(u32, u32, f32)>) -> Self {
        // compute the value of the center (just a mean)
        let (mut sum_x, mut sum_y) = (0, 0);
        let size = pixels.len() as u32;
        for (px, py, _) in &pixels {
            sum_x += px;
            sum_y += py;
        }
        sum_x /= size;
        sum_y /= size;
        let center = (sum_x, sum_y);

        // compute extremums
        // 1: min x
        // 2: max x
        // 3: min y
        // 4: max y

        let mut extremums = Vec::<(u32, u32)>::new();

        let (x1, y1, _) = pixels
            .iter()
            .min_by_key(|&&(x, _, _)| x)
            .unwrap();
        let (x2, y2, _) = pixels
            .iter()
            .max_by_key(|&&(x, _, _)| x)
            .unwrap();
        let (x3, y3, _) = pixels
            .iter()
            .min_by_key(|&&(_, y, _)| y)
            .unwrap();
        let (x4, y4, _) = pixels
            .iter()
            .max_by_key(|&&(_, y, _)| y)
            .unwrap();

        extremums.push((*x1, *y1));
        extremums.push((*x2, *y2));
        extremums.push((*x3, *y3));
        extremums.push((*x4, *y4));

        // compute dimensions
        let width = extremums[MAX_X].0 - extremums[MIN_X].0;
        let height = extremums[MAX_Y].1 - extremums[MIN_Y].1;

        // compute angle in radians
        let oposite = extremums[MAX_X].1 - extremums[MIN_X].1;
        let adjacent = extremums[MAX_X].0 - extremums[MIN_X].0;
        let tan = (oposite as f32) / (adjacent as f32);
        let angle = tan.atan();

        Self {
            pixels,
            center,
            extremums,
            width,
            height,
            mark: (1, 255, 1),
            angle,
            neighbors: [None; 4],
        }
    }

    /// Get the `width` of the shape.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the `height` of the shape.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the `size` of the shape (the number of pixels)
    pub fn size(&self) -> usize {
        self.pixels.len()
    }

    /// Get the `center` of the shape.
    pub fn center(self) -> (u32, u32) {
        self.center
    }

    /// Set a neighbor of the shape.
    ///
    /// ## Args
    ///
    /// * `neighbor` - the new neoghbor (index in the containig [`ShapeTab`])
    /// * `position` - the position of the new neighbor relativly to the current shape
    pub fn set_neighbor(&mut self, neighbor: usize, position: usize) {
        self.neighbors[position] = Some(neighbor);
    }

    /// Estimate the color of the [`Shape`] on an [Image].
    ///
    /// ## Args
    ///
    /// * `image` - the image to use to estimate the color
    ///
    /// ## Notes
    ///
    /// The estimation will be on a square of length 1/3 * size of the shape, in center of it.
    pub fn estimate_color(&self, image: &mut Image, color_data: &ColorTable) -> RgbColor {
        // compute 1/3 of the length of the square
        let delta = (self.width + self.height) / 6;

        // compute x and y
        // we use delta to get a little square in center of the shape
        // to avoid parasite pixels on the border
        let x = self.extremums[MIN_X].0 + delta;
        let y = self.extremums[MIN_Y].1 + delta;

        //image.estimate_square_color(x, y, delta)
        let shape_color = image.get_average_color(x, y, delta);

        //println!("{}", shape_color);

        let mut min_color = color_data.get_color(0);
        let mut min_dist = min_color.distance(shape_color);

        for i in 1..color_data.colors.len() {
            let color = color_data.get_color(i);

            let dist = color.distance(shape_color);

            if dist < min_dist {
                min_dist = dist;
                min_color = color;
            }
        }

        min_color

    }


    /// Compute the color of the [`Shape`] on an [Image].
    /// 
    /// ## Args
    ///
    /// * `image` - the image to use to estimate the color
    ///
    /// ## Returns
    /// 
    /// The average color of the [`Shape`].
    /// 
    /// ## Notes
    ///
    /// It will compute an average on a square of length 1/3 * size of the shape, in center of it.
    pub fn get_brut_color(&self, image: &Image) -> RgbColor {
        // compute 1/3 of the length of the square
        let delta = (self.width + self.height) / 6;

        // compute x and y
        // we use delta to get a little square in center of the shape
        // to avoid parasite pixels on the border
        let x = self.extremums[MIN_X].0 + delta;
        let y = self.extremums[MIN_Y].1 + delta;

        image.get_average_color(x, y, delta)
    }

    /// Compute the colliding factor between to shapes
    pub fn colliding_factor(&self, other: &Shape) -> f32 {
        let x_borned =
            (self.extremums[MAX_X].0 <= other.extremums[MAX_X].0 &&
                self.extremums[MAX_X].0 >= other.extremums[MIN_X].0) ||
            (self.extremums[MIN_X].0 <= other.extremums[MAX_X].0 &&
                self.extremums[MIN_X].0 >= other.extremums[MIN_X].0);
        let y_borned =
            (self.extremums[MAX_Y].1 <= other.extremums[MAX_Y].1 &&
                self.extremums[MAX_Y].1 >= other.extremums[MIN_Y].1) ||
            (self.extremums[MIN_Y].1 <= other.extremums[MAX_Y].1 &&
                self.extremums[MIN_Y].1 >= other.extremums[MIN_Y].1);

        if x_borned && y_borned {
            let coll_width =
                (self.extremums[MAX_X].0.min(other.extremums[MAX_X].0) as f32) -
                (self.extremums[MIN_X].0.max(other.extremums[MIN_X].0) as f32);
            let coll_height =
                (self.extremums[MAX_Y].1.min(other.extremums[MAX_Y].1) as f32) -
                (self.extremums[MIN_Y].1.max(other.extremums[MIN_Y].1) as f32);

            (coll_width * coll_height) / ((self.width * self.height) as f32)
        } else {
            0.0
        }
    }

    /// Merge two shapes
    pub fn merge(&mut self, other: &Shape) {
        let x_min = other.extremums[MIN_X].0;
        let x_max = other.extremums[MAX_X].0;
        let y_min = other.extremums[MIN_Y].1;
        let y_max = other.extremums[MAX_Y].1;

        for (x, y, cons) in &mut self.pixels {
            if *x >= x_min && *x <= x_max && *y >= y_min && *y <= y_max {
                if let Some(old_cons) = other.get_consistancy_at(*x, *y) {
                    *cons += old_cons + 20.0;
                    *cons = (*cons).min(255.0);
                } else {
                    *cons -= 1.0;
                }
            }
        }
    }

    /// je sais plus
    fn get_consistancy_at(&self, x: u32, y: u32) -> Option<f32> {
        for &(x_val, y_val, f_val) in &self.pixels {
            if x_val == x && y_val == y {
                return Some(f_val);
            }
        }
        None
    }

    /// Draw the [`Shape`] on an image.
    ///
    /// ## Args
    ///
    /// * `image` - the [Image] to draw on
    pub fn draw(&self, image: &mut Image) {
        let mut rng = rand::thread_rng();
        // shape is drawn in red
        let r = rng.gen_range(0..=255);
        let g = rng.gen_range(0..=255);
        let b = rng.gen_range(0..=255);
        let width = image.width();
        //let (r, _g, b) = self.mark;
        for (x, y, _cons) in &self.pixels {
            (*image).colored_bytes_mut()[((*y * width + *x) as usize) * 3] = r as u8;
            (*image).colored_bytes_mut()[((*y * width + *x) as usize) * 3 + 1] = g as u8;
            (*image).colored_bytes_mut()[((*y * width + *x) as usize) * 3 + 2] = b as u8;

            //(*image).gray_bytes_mut()[(*y * width + *x) as usize * 3] = r as u8;
            //(*image).gray_bytes_mut()[(*y * width + *x) as usize * 3 + 1] = g as u8;
            //(*image).gray_bytes_mut()[(*y * width + *x) as usize * 3 + 2] = b as u8;
        }
    }

    /// Draw the [`Shape`] on an image.
    ///
    /// ## Args
    ///
    /// * `image` - the [Image] to draw on
    pub fn draw_dbg(&self, image: &mut Image, r: u32, g: u32, b: u32) {
        let width = image.width();
        //let (r, _g, b) = self.mark;
        for (x, y, _cons) in &self.pixels {
            (*image).colored_bytes_mut()[((*y * width + *x) as usize) * 3] = r as u8;
            (*image).colored_bytes_mut()[((*y * width + *x) as usize) * 3 + 1] = g as u8;
            (*image).colored_bytes_mut()[((*y * width + *x) as usize) * 3 + 2] = b as u8;

            //(*image).gray_bytes_mut()[(*y * width + *x) as usize * 3] = r as u8;
            //(*image).gray_bytes_mut()[(*y * width + *x) as usize * 3 + 1] = g as u8;
            //(*image).gray_bytes_mut()[(*y * width + *x) as usize * 3 + 2] = b as u8;
        }
    }

    /// Check if the [`Shape`] is a square.
    pub fn is_square(&self) -> bool {
        // 1: min x
        // 2: max x
        // 3: min y
        // 4: max y

        let (x1, _y1) = self.extremums[MIN_X];
        let (x2, _y2) = self.extremums[MAX_X];
        let (_x3, y3) = self.extremums[MIN_Y];
        let (_x4, y4) = self.extremums[MAX_Y];

        let width = x2 - x1;
        let height = y4 - y3;

        // pourcentage de remplissage de la shape par rapport à son max
        let ratio = ((self.size() as f32) * 100.0) / ((width * height) as f32);

        if width * height != 0 && ratio > 70.0 {
            if
                (width.min(height) as f32) / (height.max(width) as f32) > 0.8
                //&& (y1 as i32 - y2 as i32).abs() < SHAPE_DELTA
                //&& (x3 as i32 - x4 as i32).abs() < SHAPE_DELTA
            {
                return true;
            } else {
                //println!("shape delta");
            }
        } else if width * height != 0 && ratio > 60.0 {
            //println!("{} % ", ratio);
        }

        false
    }

    /// Draw the center of the [`Shape`] on an image
    ///
    /// ## Args
    ///
    /// * `image` - the image to draw on
    pub fn draw_center(&self, image: &mut Image) {
        let (cx, cy) = self.center;
        // draw with green
        image.draw_point(cx, cy, 255)
    }

    /// Draw the rect outline of the [`Shape`] on an image
    ///
    /// ## Args
    ///
    /// * `image` - the image to draw on
    pub fn draw_rect(&self, image: &mut Image) {
        let (sx, sy) = (self.extremums[MIN_X].0, self.extremums[MIN_Y].1);
        // Note : the angle is removed
        //image.draw_rect(sx, sy, self.width, self.height, self.angle);
        image.draw_rect(sx, sy, self.width, self.height, 0.0);
    }

    /// Set the `mark` of the [`Shape`]
    ///
    /// ## Args
    ///
    /// `r,g,b` - the new mark
    pub fn set_mark(&mut self, r: u32, g: u32, b: u32) {
        self.mark = (r, g, b);
    }

    /// Draw the angles of the [`Shape`] on an image
    ///
    /// ## Args
    ///
    /// *`image` - the image to draw on
    pub fn draw_angles(&self, image: &mut Image) {
        let mut up_left = (0, 0);
        let mut dist = 0.0; // Initialize dist with a large value
        self.pixels.iter().for_each(|(x, y, _)| {
            // check if it is in quarter uper left (chai pas parler anglais déso)
            if *x < self.center.0 && *y < self.center.1 {
                let curr_dist = (
                    ((up_left.0 as f64) - (*x as f64)).powi(2) +
                    ((up_left.1 as f64) - (*y as f64)).powi(2)
                ).sqrt();
                if curr_dist > dist {
                    dist = curr_dist;
                    up_left = (*x, *y);
                }
            }
        });

        if dist != 0.0 {
            image.draw_square(up_left.0, up_left.1, 1);
        }
    }

    /// Compute the distance between the center of this [`Shape`] and another one.
    ///
    /// ## Args
    ///
    /// * `other` - the other shape
    pub fn distance(&self, other: &Shape) -> f64 {
        (
            ((self.center.0 as f64) - (other.center.0 as f64)).powi(2) +
            ((self.center.1 as f64) - (other.center.1 as f64)).powi(2)
        ).sqrt()
    }

    /// Compute the angle between the center of this [`Shape`] and another one (in radians).
    ///
    /// ## Args
    ///
    /// * `other` - the other shape
    pub fn angle(&self, other: &Shape) -> f64 {
        // compute deltas
        let dx = other.center.0 as f64 - self.center.0 as f64;
        let dy = other.center.1 as f64 - self.center.1 as f64 as f64;

        // compute the angle
        dy.atan2(dx)
    }
}
