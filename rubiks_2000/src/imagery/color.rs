//! This module contains [RgbColor], [ColorTable] and [RgbData] definitions, used for cube detection.

// std
use std::fmt;

// local
use crate::{
    critical,
    cube::{ enums::ColorTag, face_cube::FaceCube },
    error,
    gui::camera::CONV_MATRIX,
};

use super::utils::{ self, * };

/// A rgb representation of a color.
#[derive(Clone, Copy, PartialEq)]
pub struct RgbColor {
    /// The R canal
    r: u8,
    /// The G canal
    g: u8,
    /// The B canal
    b: u8,
}

impl RgbColor {
    /// Create a new [RgbColor] from rgb values
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Get the value of the R canal
    pub fn r(self) -> u8 {
        self.r
    }

    /// Get the value of the R canal
    pub fn g(self) -> u8 {
        self.g
    }

    /// Get the value of the R canal
    pub fn b(self) -> u8 {
        self.b
    }

    /// Get the RGB values as a tuple
    pub fn rgb(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    /// Get the RGB values as a normalized ([0., 1.]) tuple.
    pub fn rgb_normalized(&self) -> (f32, f32, f32) {
        ((self.r as f32) / 255.0, (self.g as f32) / 255.0, (self.b as f32) / 255.0)
    }

    /// Get the gray scale of the [RgbColor]
    pub fn gray_scale(self) -> Self {
        let gray = utils::round(
            (((self.r as u32) + (self.g as u32) + (self.b as u32)) / 3) as u8,
            GRAY_SCALE_ROUND
        );
        RgbColor::new(gray, gray, gray)
    }

    /// Get the XYZ value of the [RgbColor]
    fn rgb_to_xyz(self, r: u8, g: u8, b: u8) -> (f32, f32, f32) {
        let r = (r as f32) / 255.0;
        let g = (g as f32) / 255.0;
        let b = (b as f32) / 255.0;

        // apply gamma correction
        let r = if r > 0.04045 { ((r + 0.055) / 1.055).powf(2.4) } else { r / 12.92 };
        let g = if g > 0.04045 { ((g + 0.055) / 1.055).powf(2.4) } else { g / 12.92 };
        let b = if b > 0.04045 { ((b + 0.055) / 1.055).powf(2.4) } else { b / 12.92 };

        // convert to XYZ using the D65 illuminant
        let x = r * 0.4124 + g * 0.3576 + b * 0.1805;
        let y = r * 0.2126 + g * 0.7152 + b * 0.0722;
        let z = r * 0.0193 + g * 0.1192 + b * 0.9505;

        (x, y, z)
    }

    /// Get the LAB value from the XYZ value of the [RgbColor]
    fn xyz_to_lab(self, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        // reference white point (D65)
        let x_ref = 0.95047;
        let y_ref = 1.0;
        let z_ref = 1.08883;

        let x = x / x_ref;
        let y = y / y_ref;
        let z = z / z_ref;

        let f = |t: f32| -> f32 {
            if t > 0.008856 { t.powf(1.0 / 3.0) } else { 7.787 * t + 16.0 / 116.0 }
        };

        let l = 116.0 * f(y) - 16.0;
        let a = 500.0 * (f(x) - f(y));
        let b = 200.0 * (f(y) - f(z));

        (l, a, b)
    }

    /// Get the LAB value of the [RgbColor] as a tuple
    fn to_lab_as_tuple(self) -> (f32, f32, f32) {
        let (x, y, z) = self.rgb_to_xyz(self.r, self.g, self.b);
        self.xyz_to_lab(x, y, z)
    }

    /// Get the distance between the [RgbColor] and an other using LAB values.
    ///
    /// ## Args
    ///
    /// * `other` - the other [RgbColor] to compute the distance with
    pub fn distance(self, other: RgbColor) -> f64 {
        let square_diff = |a, b| ((a as f64) - (b as f64)).powi(2);

        let (h1, s1, v1) = self.to_lab_as_tuple();
        let (h2, s2, v2) = other.to_lab_as_tuple();

        let sum_of_squared_diffs = square_diff(h1, h2) + square_diff(s1, s2) + square_diff(v1, v2);

        sum_of_squared_diffs.sqrt()
    }

    /// Round the [RgbColor] with a given round factor.
    ///
    /// ## Args
    ///
    /// * `f` - the round factor to use
    pub fn round(&mut self, f: f32) {
        self.r = round(self.r, f);
        self.g = round(self.g, f);
        self.b = round(self.b, f);
    }
}

// display for RgbColor
impl fmt::Display for RgbColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.r, self.g, self.b)
    }
}

/// A color table containing the links between a [ColorTag] and the associated [RgbColor].
#[derive(Clone)]
pub struct ColorTable {
    /// The color table
    pub(super) colors: Vec<(RgbColor, u32, ColorTag)>,
}

impl ColorTable {
    /// Create an new empty [ColorTable]
    pub const fn new() -> Self {
        Self {
            colors: vec![],
        }
    }

    /// Get all the [RgbColor] considered as valid
    pub fn get_valid_color(&self) -> Vec<&RgbColor> {
        let valid_threshold = 20;
        self.colors
            .iter()
            .filter(|&&(_, x, _)| x > valid_threshold)
            .map(|&(ref color, _, _)| color)
            .collect()
    }

    /// Add a new [RgbColor] to the table without tag associated
    ///
    /// ## Args
    ///
    /// * `color` - the [RgbColor] to add
    pub fn add(&mut self, color: RgbColor) {
        self.colors.push((color, 0, ColorTag::None));
    }

    /// Add a new [RgbColor] to the table and set its tag
    ///
    /// ## Args
    ///
    /// * `color` - the [RgbColor] to add
    /// * `tag` - the [ColorTag] of the new color
    pub fn add_with_tag(&mut self, color: RgbColor, tag: ColorTag) {
        // if tag == ColorTag::L {
        //     tag = ColorTag::R;
        // } else if tag == ColorTag::R {
        //     tag = ColorTag::L;
        // } else if tag == ColorTag::U {
        //     tag = ColorTag::D;
        // } else if tag == ColorTag::D {
        //     tag = ColorTag::U;
        // }

        self.add(color);
        self.set_tag(color, tag);
    }

    /// Remove a entry of the table at a certain index.
    ///
    /// ## Args
    ///
    /// * `index` - the index of the entry to remove
    pub fn remove(&mut self, index: usize) {
        self.colors.remove(index);
    }

    /// Get the [RgbCoplor] of an entry at a certain index
    ///
    /// * `index` - the index of the entry to get
    pub fn get_color(&self, index: usize) -> RgbColor {
        self.colors[index].0
    }

    /// Update the consistancy of a [RgbColor] at
    pub fn update_consistancy(&mut self, index: usize) {
        self.colors[index].1 += 1;
    }

    /// Keep only the colors whose consistancy is greater than threshold.
    ///
    /// ## Args
    ///
    /// * `threshold` - the threshold to compare with each consistancy
    pub fn purge(&mut self, threshold: u32) {
        self.colors.retain(|&(_, x, _)| x >= threshold);
    }

    /// Get the corresponding [ColorTag] from a [RgbColor].
    ///
    /// ## Args
    ///
    /// * `color` - the [RgbColor] to get the tag
    pub fn tag_from_rgb(&self, color: RgbColor) -> ColorTag {
        for (rgb, _, tag) in &self.colors {
            if *rgb == color {
                return *tag;
            }
        }
        critical!("Color not found in ColorData: {}", color);
    }

    /// Get the corresponding [RgbColor] from the [ColorTag] associated.
    ///
    /// ## Args
    ///
    /// * `color_tag` - the tag to get the corresponding color
    pub fn rgb_from_tag(&self, color_tag: ColorTag) -> Option<RgbColor> {
        for (rgb, _, tag) in &self.colors {
            if *tag == color_tag {
                return Some(*rgb);
            }
        }
        None
    }

    /// Set a tag to a [RgbColor] entry.
    /// 
    /// ## Args
    /// 
    /// * `color` - the rgb color whose associated tag is changed
    /// * `tag` - the new color tag
    pub fn set_tag(&mut self, color: RgbColor, tag: ColorTag) {
        // if tag == ColorTag::L {
        //     tag = ColorTag::R;
        // } else if tag == ColorTag::R {
        //     tag = ColorTag::L;
        // } else if tag == ColorTag::U {
        //     tag = ColorTag::D;
        // } else if tag == ColorTag::D {
        //     tag = ColorTag::U;
        // }

        // remove old tag if it exists
        while let Some(tuple) = self.colors.iter_mut().find(|&&mut (_, _, t)| t == tag) {
            //println!("removed");
            tuple.2 = ColorTag::None;
        }

        // change the tag
        if let Some(tuple) = self.colors.iter_mut().find(|&&mut (rgb, _, _)| rgb == color) {
            if tuple.2 == tag || tuple.2 == ColorTag::None {
                tuple.2 = tag;
            }
        }
    }

    /// Print the color table for debug.
    pub fn print(&self) {
        for (rgb, _, tag) in &self.colors {
            println!("{} -> {}", tag, rgb);
        }
        println!();
    }
}

/// A representiation of the cube with Rgb values, used for cube detection with [ColorTable] as a transition between the cube detected and [FaceletCube].
#[derive(Clone)]
pub struct RgbData {
    /// The [RgbColor] vector and its consistancy
    pub colors: Vec<Vec<(RgbColor, u32)>>,
}

impl RgbData {
    /// Create a new [RgbData] of size 54, empty.
    pub fn new() -> Self {
        Self {
            colors: vec![vec![]; 54],
        }
    }

    /// Update the color consistancy if it exists or create it otherwise
    /// 
    /// ## Args
    /// 
    /// * `case` - the case to update the color
    /// * `color` - the color to add
    pub fn update_case(&mut self, case: usize, color: RgbColor) {
        let mut found = false;

        for c in &mut self.colors[case] {
            if c.0 == color {
                found = true;
                c.1 += 1;
                break;
            } else {
                // decrease the consistancy of the others
                if c.1 > 1 {
                    c.1 -= 4;
                }
            }
        }

        // the color doesn't exist
        if !found {
            // so we create it
            self.colors[case].push((color, 1));
        }
    }

    /// Force set a [RgbColor] at a certain case.
    /// 
    /// ## Args
    /// 
    /// * `case` - the case where to set the color
    /// * `color` - the color to ser
    pub fn set(&mut self, case: usize, color: RgbColor) {
        self.colors[case].push((color, u32::MAX));
    }

    /// Get the more consistant [RgbColor] of acertain case.
    /// 
    /// ## Args
    /// 
    /// *`case` - the case where to get the max color
    pub fn get_max(&self, case: usize) -> Option<RgbColor> {
        if let Some((color, _)) = self.colors[case].iter().max_by_key(|&(_, value)| value) {
            return Some(*color);
        } else {
            return None;
        }
    }

    /// Convert the [RgbData] into a [FaceCube] by using a [ColorTable] for the transition.
    /// 
    /// ## Args
    /// 
    /// * `color_table` - the [ColorTable] use for transition
    pub fn to_facelet_cube(&self, color_table: &ColorTable) -> FaceCube {
        let mut fcube = FaceCube::new();

        for (i, v) in self.colors.iter().enumerate() {
            if v.len() == 0 {
                error!("Rgb data is not filled at {}", i);
            } else {
                let (rgb, _) = v[0];
                let tag = color_table.tag_from_rgb(rgb);

                if tag == ColorTag::None {
                    error!("Color table is not filled : {}", tag);
                }

                let transposed = CONV_MATRIX[i];

                fcube.set_facelet(transposed, tag);
            }
        }

        fcube
    }
}
