use std::cmp;

use crate::{
    imagery::{color::*, shape::*, shape_tab::*},
    warning,
};

/// An image with usefull functions to detect squares.
///
/// ## Fields
///
/// * `colored_bytes` - the colored image as bytes
/// * `gray_bytes` - the image in grayscale as bytes
/// * `width` - the width of the image
/// * `height` - the height of the image
#[derive(Clone)]
pub struct Image {
    /// The colored image as bytes
    colored_bytes: Vec<u8>,
    /// The image in grayscale as bytes
    gray_bytes: Vec<u8>,
    // The width of the image
    width: u32,
    /// The height of the image
    height: u32,
}

impl Image {
    // -------------- ------ ------------------
    // -------------- BASICS ------------------
    // -------------- ------ ------------------

    /// Create a new [`Image`] from a camera frame.
    ///
    /// ## Args
    ///
    ///  * `frame` - the camera frame
    ///
    /// ## Returns
    ///
    /// The [`Image`] created.
    ///
    /// ## Notes
    ///
    /// `gray_bytes` and `colored_bytes` are both initialized.
    pub fn new_frome_frame(frame: &rscam::Frame) -> Self {
        // get dimentions
        let width = frame.resolution.0;
        let height = frame.resolution.1;

        // create vecs for bytes
        let mut gray_bytes = vec![];
        let mut colored_bytes = vec![];

        // convert the frame in a vec
        let frame_vec = frame.to_vec();
        for x in 0..width {
            for y in 0..height {
                let index = ((x * height + y) * 3) as usize;
                let (r, g, b) = (frame_vec[index], frame_vec[index + 1], frame_vec[index + 2]);
                // convert into gray
                let gray = (((r as u16) + (g as u16) + (b as u16)) / 3) as u8;
                gray_bytes.push(gray);
                gray_bytes.push(gray);
                gray_bytes.push(gray);
                colored_bytes.push(r);
                colored_bytes.push(g);
                colored_bytes.push(b);
            }
        }

        Self {
            colored_bytes: colored_bytes,
            gray_bytes: gray_bytes,
            width: width,
            height: height,
        }
    }

    /// Clear an [`Image`] by setting all its collored bytes to black.
    pub fn clear(&mut self) {
        for byte in self.colored_bytes.iter_mut() {
            *byte = 0;
        }
    }

    /// Get the `width` of the [`Image`].
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the `height` of the [`Image`].
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the `gray_bytes` of the [`Image`].
    pub fn gray_bytes(&self) -> &Vec<u8> {
        &self.gray_bytes
    }

    /// Get the `gray_bytes` of the [`Image`] muttably.
    pub fn gray_bytes_mut(&mut self) -> &mut Vec<u8> {
        &mut self.gray_bytes
    }

    /// Get the `colored_bytes` of the [`Image`].
    pub fn colored_bytes(&self) -> &Vec<u8> {
        &self.colored_bytes
    }

    /// Get the `colored_bytes` of the [`Image`] muttably.
    pub fn colored_bytes_mut(&mut self) -> &mut Vec<u8> {
        &mut self.colored_bytes
    }

    /// Mirror an image on the vertical axe
    ///
    /// ## Notes
    ///
    /// Will affect both `gray_bytes` and `colored_bytes`
    pub fn mirror_horizontal(&mut self) {
        let row_size = (self.width * 3) as usize; // 3 u8 values per pixel (RGB)
        let width = self.width as usize;

        for y in 0..self.height as usize {
            for x in 0..(self.width / 2) as usize {
                for c in 0..3 {
                    // Swap R, G, B separately
                    self.colored_bytes.swap(
                        y * row_size + x * 3 + c,
                        y * row_size + (width - x - 1) * 3 + c,
                    );
                    self.gray_bytes.swap(
                        y * row_size + x * 3 + c,
                        y * row_size + (width - x - 1) * 3 + c,
                    );
                }
            }
        }
    }

    // -------------- ------- ------------------
    // -------------- FILTERS ------------------
    // -------------- ------- ------------------

    /// Apply a sobel edge detection filter to the [`Image`].
    ///
    /// ## Note
    ///
    /// * Will only affect the `gray_bytes`.
    /// * All the edges are set in **gray** (depending on their magnitude), others in **black**.
    pub fn apply_sobel_filter(&mut self) {
        let width = self.width as usize;
        let height = self.height as usize;
        let mut result = vec![0; self.gray_bytes.len()];

        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let mut gx = 0;
                let mut gy = 0;

                let index = (y * width + x) * 3;

                // apply sobel filter with kernel
                gx += -(self.gray_bytes[index - width * 3 - 3] as i32)
                    + (self.gray_bytes[index + width * 3 - 3] as i32);
                gx += -2 * (self.gray_bytes[index - 3] as i32)
                    + 2 * (self.gray_bytes[index + 3] as i32);
                gx += -(self.gray_bytes[index - width * 3 + 3] as i32)
                    + (self.gray_bytes[index + width * 3 + 3] as i32);

                gy += -(self.gray_bytes[index - width * 3 - 3] as i32)
                    - 2 * (self.gray_bytes[index - width * 3] as i32)
                    - (self.gray_bytes[index - width * 3 + 3] as i32);
                gy += (self.gray_bytes[index + width * 3 - 3] as i32)
                    + 2 * (self.gray_bytes[index + width * 3] as i32)
                    + (self.gray_bytes[index + width * 3 + 3] as i32);

                // gradiant value
                let magnitude = (gx.abs() + gy.abs()) as u8;

                result[index] = magnitude;
                result[index + 1] = magnitude;
                result[index + 2] = magnitude;
            }
        }

        self.gray_bytes = result;
    }

    /// Apply a median convolutionnal filter on the [`Image`]
    ///
    /// ## Args
    ///
    /// * `n` - the size of the kernel to use
    ///
    /// ## Notes
    ///
    /// * Only affect `gray_bytes`
    /// * Take the median of all the pixels around to a distance n
    /// and set this value to each pixel
    pub fn apply_median_filter(&mut self, n: usize) {
        let mut result = self.gray_bytes.clone();

        // kernel size
        let window_size = n * n;
        let mut neighborhood = vec![0; window_size];

        let width = self.width as usize;
        let height = self.height as usize;

        for y in 0..height {
            for x in 0..width {
                let mut index = 0;

                // go through the neighbours
                for j in 0..n {
                    for i in 0..n {
                        let nx = cmp::min(
                            cmp::max(0, (x as isize) + (i as isize) - ((n / 2) as isize)),
                            (width - 1) as isize,
                        ) as usize;
                        let ny = cmp::min(
                            cmp::max(0, (y as isize) + (j as isize) - ((n / 2) as isize)),
                            (height - 1) as isize,
                        ) as usize;

                        let byte_index = (ny * width + nx) * 3;
                        neighborhood[index] = self.gray_bytes[byte_index] as u32;
                        index += 1;
                    }
                }

                // sort it
                neighborhood.sort_unstable();
                // get the median
                let median_value = neighborhood[window_size / 2] as u8;

                let result_index = (y * width + x) * 3;
                result[result_index] = median_value;
                result[result_index + 1] = median_value;
                result[result_index + 2] = median_value;
            }
        }

        self.gray_bytes = result;
    }

    /// Apply a convolutionnal blur filter on the [`Image`].
    ///
    /// ## Args
    ///
    /// `mat` - the convolutionnal kernel
    ///
    /// ## Notes
    ///
    /// * Will only affect `gray_bytes`.
    /// * Take the weighted mean of all pixels around and set this value for each pixel.
    pub fn apply_blur_filter(&mut self, kernel: &[f32]) {
        let width = self.width as usize;
        let height = self.height as usize;
        // res
        let mut new_bytes = self.gray_bytes.clone();

        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let mut sum = 0.0;
                let mut divisor = 0.0;

                // apply this to each canal
                for j in 0..3 {
                    for k in 0..3 {
                        // neighbour index
                        let index = ((y + k - 1) * width + (x + j - 1)) * 3 + j;
                        // weighted sum
                        sum += f64::from(self.gray_bytes[index]) * f64::from(kernel[k * 3 + j]);
                        // coefficient sum
                        divisor += f64::from(kernel[k * 3 + j]);
                    }
                }

                let new_value = (sum / divisor) as u8;
                // index of the pixel in byte vector
                let index = (y * width + x) * 3;
                new_bytes[index..index + 3].copy_from_slice(&[new_value; 3]);
            }
        }
        self.gray_bytes = new_bytes;
    }

    /// Binarize the [`Image`] with a certain threshold.
    ///
    /// ## Args
    ///
    /// * `threshold` - the treshold to use
    ///
    /// ## Note
    ///
    /// * Will only affect `gray_bytes`
    /// * Set all pixel lower than the threshold in black
    pub fn apply_threshold(&mut self, threshold: u8) {
        for pixel in self.gray_bytes.iter_mut() {
            *pixel = if *pixel > threshold { *pixel } else { 0 };
        }
    }

    /// Apply contrast to an [`Image`].
    ///
    /// ## Args
    ///
    /// * `factor` - the contrast factor
    ///
    /// ## Notes
    ///
    /// Will only affect `gray_bytes`
    pub fn apply_contrast(&mut self, factor: f64) {
        // compute average
        let mut sum: f64 = 0.0;
        for byte in &self.gray_bytes {
            sum += *byte as f64;
        }
        let mean = sum / (self.gray_bytes.len() as f64);

        // update each pix value
        for byte in &mut self.gray_bytes {
            let diff = (*byte as f64) - mean;
            *byte = (mean + diff * factor) as u8;
        }
    }

    /// Apply contrast to an [`Image`].
    ///
    /// ## Args
    ///
    /// * `factor` - the contrast factor
    ///
    /// ## Notes
    ///
    /// Will only affect `colored_bytes`
    pub fn apply_colored_contrast(&mut self, factor: f64) {
        // compute average
        let l = self.colored_bytes.len();
        for i in (0..l).step_by(3) {
            let r = self.colored_bytes[i] as f64;
            let g = self.colored_bytes[i + 1] as f64;
            let b = self.colored_bytes[i + 2] as f64;

            let gray = (r + g + b) / 3.0;
            let a = 255.0 * (factor * (gray / 255.0).ln()).exp();

            let nr = (0_f64).max((255_f64).min(r + a - gray));
            let ng = (0_f64).max((255_f64).min(g + a - gray));
            let nb = (0_f64).max((255_f64).min(b + a - gray));

            self.colored_bytes[i] = nr as u8;
            self.colored_bytes[i + 1] = ng as u8;
            self.colored_bytes[i + 2] = nb as u8;
        }
    }

    /// Applys mass detection to the [`Image`]
    ///
    /// ## Returns
    ///
    /// The shapes detected.
    ///
    /// ## Note
    ///
    /// Will only read `gray_bytes`.
    pub fn apply_mass_detection(&mut self) -> ShapeTab {
        let n_width = self.width;
        let n_height = self.height;

        let mut marqued: Vec<Vec<bool>> = vec![vec![false; n_height as usize]; n_width as usize];

        let mut shape_tab = ShapeTab::new();

        for px in 0..n_width {
            for py in 0..n_height {
                // normally, the image is binarized so we create a mass only on the black
                if marqued[px as usize][py as usize]
                    || self.gray_bytes[((py * self.width + px) * 3) as usize] > 0
                {
                    continue;
                }

                let mut pix_queue = vec![];
                let mut pix_shape_tab: Vec<(u32, u32, f32)> = vec![];
                pix_queue.push((px as i32, py as i32));
                pix_shape_tab.push((px, py, 0.0));

                marqued[px as usize][py as usize] = true;

                while pix_queue.len() > 0 {
                    let (x, y) = pix_queue.pop().unwrap();

                    for i in x - 1..x + 2 {
                        if i < 0 || i >= (n_width as i32) {
                            continue;
                        }

                        for j in y - 1..y + 2 {
                            if j < 0 || j >= (n_height as i32) || marqued[i as usize][j as usize] {
                                continue;
                            }

                            let color = self.gray_bytes
                                [(((j as u32) * self.width + (i as u32)) as usize) * 3];
                            if color == 0 {
                                pix_queue.push((i, j));
                                pix_shape_tab.push((i as u32, j as u32, 0.0));
                                marqued[i as usize][j as usize] = true;
                            }
                        }
                    }
                }

                let shape = Shape::new(pix_shape_tab.clone());
                //shape.add((px, py), 0.);
                shape_tab.add(shape);
            }
        }

        shape_tab
    }

    // -------------- ---- ------------------
    // -------------- DRAW ------------------
    // -------------- ---- ------------------

    /// Draw a green point on the [`Image`].
    ///
    /// ## Args
    ///
    /// * `x,y` - the position of the point.
    /// * `color` - the color of the point on the green cannal
    ///
    /// ## Notes
    ///
    /// Will only affect `colored_bytes`
    pub fn draw_point(&mut self, x: u32, y: u32, color: u8) {
        let x_start = x.saturating_sub(2) as usize;
        let y_start = y.saturating_sub(2) as usize;
        let x_end = (x + 2).min(self.width - 1) as usize;
        let y_end = (y + 2).min(self.height - 1) as usize;

        for j in y_start..=y_end {
            for i in x_start..=x_end {
                let index = (j * (self.width as usize) + i) * 3;
                self.colored_bytes[index + 1] = color;
            }
        }
    }

    /// Draw a green line on the [`Image`].
    ///
    /// ## Args
    ///
    /// * `x1, y1` - the position of the starting point
    /// * `x2, y2` - the position of the ending point
    ///
    /// ## Panics
    ///
    /// If one of the points is out ouf the image bounds.
    ///
    /// ## Notes
    ///
    /// Will only modify `colored_bytes`
    pub fn draw_line(&mut self, x1: u32, y1: u32, x2: u32, y2: u32) {
        if x1 >= self.width || y1 >= self.height || x2 >= self.width || y2 >= self.height {
            panic!("Line coords out of bounds");
        }

        let index = |x: u32, y: u32| y * self.width * 3 + x * 3;

        let dx = (x2 as isize) - (x1 as isize);
        let dy = (y2 as isize) - (y1 as isize);

        // if la pente est raide ou pas
        let steep = dy.abs() > dx.abs();

        // if it is, swap
        let (mut x, mut y, dx, dy) = if steep {
            (y1 as isize, x1 as isize, dy, dx)
        } else {
            (x1 as isize, y1 as isize, dx, dy)
        };

        // determine increment by x and y
        let increment_x = if dx < 0 { -1 } else { 1 };
        let increment_y = if dy < 0 { -1 } else { 1 };

        // abs
        let dx = dx.abs();
        let dy = dy.abs();

        let max_steps = if dx > dy { dx } else { dy };

        // compute initial error
        let mut error = max_steps / 2;

        // draw
        for _ in 0..=max_steps {
            let pixel_index = if steep {
                index(y as u32, x as u32) as usize
            } else {
                index(x as u32, y as u32) as usize
            };

            // draw in green
            //self.gray_bytes[pixel_index] = 0;
            //self.gray_bytes[pixel_index + 1] = 255;
            //self.gray_bytes[pixel_index + 2] = 0;
            self.colored_bytes[pixel_index] = 0;
            self.colored_bytes[pixel_index + 1] = 255;
            self.colored_bytes[pixel_index + 2] = 0;

            // update err
            error -= dy;

            // if erro < 0 go toward y
            if error < 0 {
                y += increment_y;
                error += dx;
            }

            // go toward x
            x += increment_x;
        }
    }

    /// Draw an empty green square on the [`Image`].
    ///
    /// ## Args
    ///
    /// * `x, y` - the position of the left up angle of the square
    /// * `length` - the length of the side of the square
    ///
    /// ## Notes
    ///
    /// Will only affect `colored_bytes`.
    pub fn draw_square(&mut self, x: u32, y: u32, length: u32) {
        self.draw_line(x, y, x, y + length);
        self.draw_line(x, y, x + length, y);
        self.draw_line(x + length, y, x + length, y + length);
        self.draw_line(x, y + length, x + length, y + length);
    }

    ///  Draw an empty rectangle on the [`Image`].
    ///
    /// ## Args
    ///
    /// * `x,y` - the center of the rectangle
    /// * `width, height` - the dimensions of the rectangle
    /// * `angle` - the angle of the rectangle
    ///
    /// ## Notes
    ///
    /// Will only affect `colored_bytes`.
    pub fn draw_rect(&mut self, x: u32, y: u32, width: u32, height: u32, angle: f32) {
        let tan = angle.tan();

        // basic trigo
        // dy = delta to use on y
        // dx = delta to use on x
        let dx = (tan * (width as f32)) as u32;
        let dy = (tan * (height as f32)) as u32;

        self.draw_line(x + dx, y, x, y + height - dy); // left side
        self.draw_line(x + dx, y, x + width, y + dy); // up side
        self.draw_line(x + width, y + dy, x + width - dx, y + height); // right side
        self.draw_line(x, y + height - dy, x + width - dx, y + height); // down side
    }

    /// Compute the average color in a square of the `Image`
    ///
    /// ## Args
    ///
    /// * `x,y` - the left up corner of the square
    /// * `length` - the length of the square
    ///
    /// ## Notes
    ///
    /// * Will only read `colored_bytes`.
    pub fn get_average_color(&self, x: u32, y: u32, length: u32) -> RgbColor {
        if x >= self.width() || y >= self.height() {
            panic!("Coordonnées en dehors des limites de l'self");
        }

        // convert (x,y) in index
        let red = |x: u32, y: u32| {
            let index = y * self.width() * 3 + x * 3 + 0;
            self.colored_bytes[index as usize]
        };

        let green = |x: u32, y: u32| {
            let index = y * self.width() * 3 + x * 3 + 1;
            self.colored_bytes[index as usize]
        };

        let blue = |x: u32, y: u32| {
            let index = y * self.width() * 3 + x * 3 + 2;
            self.colored_bytes[index as usize]
        };

        let (mut r, mut g, mut b) = (0.0, 0.0, 0.0);

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

        RgbColor::new(r as u8, g as u8, b as u8)
    }

    ///
    pub fn apply_color_detection(
        &mut self,
        color_data: &mut ColorTable,
        rgb_data: &mut RgbData,
        face: usize,
    ) {
        // increase contrast
        self.apply_contrast(2.0);

        // matrice for the blur filter
        let mat = [0.0, 4.0, 0.0, 4.0, 0.0, 4.0, 0.0, 4.0, 0.0];

        // blur the image
        self.apply_blur_filter(&mat);

        // perform sobel filter
        self.apply_sobel_filter();
        // binarize the image
        self.apply_threshold(30);

        // perform mass detection
        let mut shape_tab = self.apply_mass_detection();
        shape_tab.sort_squares();

        if shape_tab.size() > 9 {
            warning!("To much squares detected!!");
            //return None;
        } else if shape_tab.size() < 9 {
            if shape_tab.size() != 0 {
                //warning!("Not enough squares detected!!");
            }
        } else {
            shape_tab.init_graph_shape();
            shape_tab.estimate_graph_color(self, color_data, rgb_data, face);

            shape_tab.draw_shape_rect(self);
        }
    }

    pub fn calibrate_color_data(&mut self, color_data: &mut ColorTable) {
        self.apply_contrast(2.0);

        // matrice for the blur filter
        let mat = [0.0, 4.0, 0.0, 4.0, 0.0, 4.0, 0.0, 4.0, 0.0];

        // blur filter
        self.apply_blur_filter(&mat);

        // sobel edge detection
        self.apply_sobel_filter();
        self.apply_threshold(30);

        let mut shape_tab = self.apply_mass_detection();

        shape_tab.sort_squares();

        if shape_tab.size() > 9 {
            warning!("To much square detected during calibration!!");
        } else {
            shape_tab.calibrate_color_data(self, color_data);

            // for c in 0..color_data.colors.len() {

            //     println!("{} | {}", color_data.get_color(c), color_data.colors[c].1);

            // }

            //

            // if color_data.colors.len() > 0 {

            //     println!();

            // }

            shape_tab.draw_shape_rect(self);
        }
    }
}
