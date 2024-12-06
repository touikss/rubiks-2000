//! This module contains the [`ShapeTab`] definition.
//! [`ShapeTab`] is used to contains [`Shape`] detected and to manipulate them.

// std
use std::f64::consts::PI;

// local
use crate::warning;
use super::{ color::{ ColorTable, RgbData }, image::Image, shape::Shape, utils::* };

/// A tab containing shapes
///
/// ## Args
///
/// * `shapes` - the shapes of the tab
#[derive(Clone)]
pub struct ShapeTab {
    /// The shapes of the tab
    pub(super) shapes: Vec<Shape>,
}

impl ShapeTab {
    /// Create a new [`ShapeTab`]
    ///
    /// ## Notes
    ///
    /// `shapes` is empty.
    pub fn new() -> Self {
        Self { shapes: vec![] }
    }

    /// Add a new [`Shape`] to the tab.
    ///
    /// ## Args
    ///
    /// * `shape` - the shape to add
    pub fn add(&mut self, shape: Shape) {
        self.shapes.push(shape);
    }

    /// Remove a new [`Shape`] to the tab.
    ///
    /// ## Args
    ///
    /// * `shape` - the shape to remove
    pub fn remove(&mut self, shape: &Shape) {
        self.shapes.retain(|s| s != shape);
    }

    /// Empty the [`ShapeTab`]
    pub fn clear(&mut self) {
        self.shapes.clear();
    }

    /// Get the number of [`Shape`].
    pub fn size(&self) -> usize {
        self.shapes.len()
    }

    /// Keep only square [`Shape`].
    pub fn sort_squares(&mut self) {
        let mut squares = vec![];

        // first trim
        for shape in self.shapes.clone() {
            if shape.size() > 200 && shape.size() < 15500 && shape.is_square() {
                squares.push(shape);
            }
        }

        self.shapes.clear();

        // second trim
        for (i, square) in squares.clone().into_iter().enumerate() {
            let mut dist_avg_vec = vec![];

            for (j, other) in squares.clone().into_iter().enumerate() {
                if i != j {
                    let (x1, y1) = square.clone().center();
                    let (x2, y2) = other.center();
                    let dist = (
                        ((x2 as f64) - (x1 as f64)).powi(2) + ((y2 as f64) - (y1 as f64)).powi(2)
                    ).sqrt();
                    dist_avg_vec.push(dist);
                }
            }

            dist_avg_vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let closers_vec = dist_avg_vec.iter().take(2).cloned().collect::<Vec<_>>();

            let avg = calculate_mean(&closers_vec);
            let std_deviation = calculate_std_dev(&closers_vec);

            let ratio = (std_deviation * 100.0) / avg;

            if ratio < 20.0 {
                //println!("ratio : {} % ", ratio);
                self.shapes.push(square);
            }
        }
    }

    /// je sais plus
    pub fn merge_shapes(&mut self, mut other: ShapeTab) {
        for shape in &mut self.shapes {
            let mut coll_shapes_vec = vec![];

            // get all colliding shapes
            for other_shape in other.shapes.clone() {
                let coll_fact = shape.colliding_factor(&other_shape);
                // insert other shape in vec of colliding shape if it is
                // the insertion is sorted
                if coll_fact > 0.0 {
                    //println!("{}", coll_fact);
                    let index = coll_shapes_vec
                        .iter()
                        .position(|&(_, v)| v <= coll_fact)
                        .unwrap_or(coll_shapes_vec.len());
                    coll_shapes_vec.insert(index, (other_shape, coll_fact));
                }
            }

            if coll_shapes_vec.len() > 0 {
                let (most_coll_shape, _) = coll_shapes_vec[0].clone();
                shape.merge(&most_coll_shape);
                other.remove(&most_coll_shape);
            }
        }

        for other_shape in other.shapes {
            let mut dist_avg_vec = vec![];

            for shape in &mut self.shapes {
                let (x1, y1) = other_shape.clone().center();
                let (x2, y2) = shape.clone().center();
                let dist = (
                    ((x2 as f64) - (x1 as f64)).powi(2) + ((y2 as f64) - (y1 as f64)).powi(2)
                ).sqrt();

                let index = dist_avg_vec
                    .iter()
                    .position(|&(_, v)| v >= dist)
                    .unwrap_or(dist_avg_vec.len());
                dist_avg_vec.insert(index, (shape.clone(), dist));
            }

            if dist_avg_vec.len() > 0 {
                let (mut nearest_shape, min_dist) = dist_avg_vec[0].clone();
                if min_dist > 10.0 {
                    nearest_shape.set_mark(1, 1, 255);
                    self.add(nearest_shape);
                }
            }
        }
    }

    /// Draw the shapes on an image.
    ///
    /// ## Args
    ///
    /// * `image` - the image to draw on
    pub fn draw_shapes(&self, image: &mut Image) {
        for shape in &self.shapes {
            shape.draw(image);
        }
    }

    /// Draw the shapes centers on an image.
    ///
    /// ## Args
    ///
    /// * `image` - the image to draw on
    pub fn draw_shape_center(&self, image: &mut Image) {
        for shape in &self.shapes {
            shape.draw_center(image);
        }
    }

    /// Draw the shapes rect outlines on an image.
    ///
    /// ## Args
    ///
    /// * `image` - the image to draw on
    pub fn draw_shape_rect(&self, image: &mut Image) {
        for shape in &self.shapes {
            shape.draw_rect(image);
        }
    }

    /// Draw the shapes angles on an image.
    ///
    /// ## Args
    ///
    /// * `image` - the image to draw on
    pub fn draw_shape_angles(&mut self, image: &mut Image) {
        for shape in &self.shapes {
            shape.draw_angles(image);
        }
    }

    /// Compute each neighbor of each shape of the tab.
    pub fn init_graph_shape(&mut self) {
        let pi_4 = PI / 4.0;

        let shapes_clone = self.shapes.clone();

        for shape in &mut self.shapes {
            for (other_index, other) in shapes_clone.iter().enumerate() {
                let angle = shape.angle(&other);
                if shape.center.0 != other.center.0 || shape.center.1 != other.center.1 {
                    let mut side = LEFT;

                    // right neighbor
                    if angle <= pi_4 && angle >= -pi_4 {
                        side = RIGHT;
                    } else if
                        // up
                        angle <= -pi_4 &&
                        angle >= -3.0 * pi_4
                    {
                        side = UP;
                    } else if
                        // down
                        angle <= 3.0 * pi_4 &&
                        angle >= pi_4
                    {
                        side = DOWN;
                    }

                    // if there is already a neghbor at this side
                    if let Some(neighbor) = shape.neighbors[side] {
                        // if this one is nearest then change it
                        let dist = shape.distance(&other);
                        if shape.distance(&shapes_clone[neighbor]) > dist {
                            shape.neighbors[side] = Some(other_index);
                        }
                    } else {
                        shape.neighbors[side] = Some(other_index);
                    }
                }
            }
        }
    }

    ///
    /// This function recursively draws lines connecting shapes in a graph structure on a given image.
    ///
    /// # Arguments
    ///
    /// * `image` - A mutable reference to the `Image` on which the graph will be drawn.
    /// * `i` - The index of the current shape in the `shapes` vector.
    /// * `m` - A mutable reference to an array of booleans that tracks which shapes have been visited.
    pub fn draw_graph(&self, image: &mut Image, i: usize, m: &mut [bool; 9]) {
        m[i] = true;

        let dir = [RIGHT, UP, DOWN, LEFT];

        for d in dir {
            let shape = &self.shapes[i];
            if let Some(neighbor) = shape.neighbors[d] {
                let other = &self.shapes[neighbor];
                image.draw_line(shape.center.0, shape.center.1, other.center.0, other.center.1);
                if !m[neighbor] {
                    self.draw_graph(image, neighbor, m);
                }
            }
        }
    }

    /// Get the root of the detected graph (the left top corner).
    ///
    /// # Returns
    ///
    /// The index of the root shape in the `shapes` vector.
    pub fn get_root(&self) -> usize {
        // marqued vect to trace which shape is already checked
        let mut marqued = [false; 9];
        // the left top shape
        let mut root = 0;

        marqued[root] = true;

        while let Some(left) = self.shapes[root].neighbors[LEFT] {
            if !marqued[left] {
                root = left;
                marqued[root] = true;
            } else {
                break;
            }
        }

        while let Some(up) = self.shapes[root].neighbors[UP] {
            if !marqued[up] {
                root = up;
                marqued[root] = true;
            } else {
                break;
            }
        }

        root
    }

    /// Estimate each color of the case composing the graphe detected and store it.
    /// ## Args
    ///
    /// * `image` - A mutable reference to the `Image` object.
    /// * `color_table` - A mutable reference to the `ColorTable` object.
    /// * `rgb_data` - A mutable reference to the `RgbData` object.
    /// * `face` - The index of the current shape in the `shapes` vector.
    pub fn estimate_graph_color(
        &self,
        image: &mut Image,
        color_table: &mut ColorTable,
        rgb_data: &mut RgbData,
        face: usize
    ) {
        if self.size() != 9 {
            warning!("Shape Tab provided to graph color estimation has not a size of 9!");
            return;
        }

        // get the root of the graph
        let mut current = self.get_root();

        // draw the graph
        self.draw_graph(image, current, &mut [false; 9]);

        // set first color estimation
        rgb_data.update_case(9 * face, self.shapes[current].estimate_color(image, color_table));

        // order of parkour of the graph
        let order = [
            (0, 1, RIGHT),
            (0, 2, RIGHT),
            (1, 2, DOWN),
            (1, 1, LEFT),
            (1, 0, LEFT),
            (2, 0, DOWN),
            (2, 1, RIGHT),
            (2, 2, RIGHT),
        ];

        // if the graph is complete or not
        let mut complete = true;

        // go through the graph following order
        for (x, y, dir) in order {
            if let Some(next) = self.shapes[current].neighbors[dir] {
                // update rgb data with estimated color
                rgb_data.update_case(
                    x * 3 + y + 9 * face,
                    self.shapes[next].estimate_color(image, color_table)
                );
                // go to the next one
                current = next;
            } else {
                // the graph is nt complete (should not append since there is 9 squares)
                // sot it would mean the graph initialization failed
                complete = false;
                break;
            }
        }

        if !complete {
            warning!("Graph provided not complete, the graph initialization propably failed.");
        }
    }

    /// Calibrate color data based on the shapes detected in the image.
    ///
    /// ## Args
    ///
    /// * `image` - The image containing the detected shapes.
    /// * `color_data` - The `ColorTable` to be calibrated.
    pub fn calibrate_color_data(&self, image: &Image, color_data: &mut ColorTable) {
        let threshold = 20.0;

        for shape in &self.shapes {
            let shape_color = shape.get_brut_color(image);

            let mut min_dist = f64::MAX;
            let mut min_color = None;
            for color_index in 0..color_data.colors.len() {
                let dist = color_data.get_color(color_index).distance(shape_color);

                if dist < threshold && dist < min_dist {
                    min_dist = dist;
                    min_color = Some(color_index);
                }
            }

            let len = color_data.colors.len();

            if let Some(color) = min_color {
                color_data.update_consistancy(color);
            } else if len < 6 {
                color_data.add(shape_color);
                color_data.update_consistancy(len);
            } else {
                //warning!("There is too much color on the cube.");
            }
        }
    }
}
