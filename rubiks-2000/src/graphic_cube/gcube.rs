use std::{
    borrow::BorrowMut,
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
    usize,
};

use crate::cube::cube::Cube;
use kiss3d::{
    camera::Camera,
    nalgebra::{Isometry3, Translation3, UnitQuaternion},
    scene::SceneNode,
    window::Window,
};
use rand::Rng;

use super::gtk_binding::*;

const SIZE: f32 = 1.0;
const OFFSET: f32 = 0.2;
pub const ANIMATION_TIME: f32 = 0.4;
const SHUFFLE_ANIMATION_TIME: f32 = 0.1;

#[derive(PartialEq, Clone, Copy)]
pub enum Move {
    V,
    Vp,
    H,
    Hp,
    L,
    Lp,
}

pub fn inv_move(mv: Move) -> Move {
    let res = match mv {
        Move::V => Move::Vp,
        Move::Vp => Move::V,
        Move::H => Move::Hp,
        Move::Hp => Move::H,
        Move::L => Move::Lp,
        _ => Move::L
    };
    res
}

pub fn mv_to_string(mv: Move, n: usize) -> String {
    let res = match (mv, n) {
        (Move::V, 0) => String::from("Right"),
        (Move::V, 2) => String::from("Left"),
        (Move::L, 0) => String::from("Front"),
        (Move::L, 2) => String::from("Back"),
        (Move::H, 0) => String::from("Down"),
        (Move::H, 2) => String::from("Up"),
        (Move::Vp, 0) => String::from("Right'"),
        (Move::Vp, 2) => String::from("Left'"),
        (Move::Lp, 0) => String::from("Front'"),
        (Move::Lp, 2) => String::from("Back'"),
        (Move::Hp, 0) => String::from("Down'"),
        (Move::Hp, 2) => String::from("Up'"),
        _ => String::from("Move")
    };
    res
}

#[derive(Clone)]
struct Cublet {
    node: SceneNode,
    front_face: SceneNode,
    back_face: SceneNode,
    left_face: SceneNode,
    right_face: SceneNode,
    top_face: SceneNode,
    bottom_face: SceneNode,
    position: (usize, usize, usize),
}

impl Cublet {
    pub fn new() -> Self {
        let mut cubelet = SceneNode::new_empty();

        // Create faces
        let mut front_face = cubelet.add_quad(SIZE, SIZE, 1, 1);
        let mut back_face = cubelet.add_quad(SIZE, SIZE, 1, 1);
        let mut right_face = cubelet.add_quad(SIZE, SIZE, 1, 1);
        let mut left_face = cubelet.add_quad(SIZE, SIZE, 1, 1);
        let mut top_face = cubelet.add_quad(SIZE, SIZE, 1, 1);
        let mut bottom_face = cubelet.add_quad(SIZE, SIZE, 1, 1);

        // Set faces positions
        let half_size = SIZE / 2.0;
        back_face.set_local_translation(Translation3::new(0.0, 0.0, half_size));
        front_face.set_local_translation(Translation3::new(0.0, 0.0, -half_size));
        right_face.set_local_translation(Translation3::new(-half_size, 0.0, 0.0));
        left_face.set_local_translation(Translation3::new(half_size, 0.0, 0.0));
        top_face.set_local_translation(Translation3::new(0.0, half_size, 0.0));
        bottom_face.set_local_translation(Translation3::new(0.0, -half_size, 0.0));

        // Set faces color to blac
        back_face.set_color(0.0, 0.0, 0.0);
        front_face.set_color(0.0, 0.0, 0.0);
        left_face.set_color(0.0, 0.0, 0.0);
        right_face.set_color(0.0, 0.0, 0.0);
        top_face.set_color(0.0, 0.0, 0.0);
        bottom_face.set_color(0.0, 0.0, 0.0);

        // Apply face rotation
        back_face.set_local_rotation(UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0));
        front_face.set_local_rotation(UnitQuaternion::from_euler_angles(
            0.0,
            std::f32::consts::PI,
            0.0,
        ));
        right_face.set_local_rotation(UnitQuaternion::from_euler_angles(
            0.0,
            std::f32::consts::FRAC_PI_2,
            0.0,
        ));
        left_face.set_local_rotation(UnitQuaternion::from_euler_angles(
            0.0,
            -std::f32::consts::FRAC_PI_2,
            0.0,
        ));
        top_face.set_local_rotation(UnitQuaternion::from_euler_angles(
            -std::f32::consts::FRAC_PI_2,
            0.0,
            0.0,
        ));
        bottom_face.set_local_rotation(UnitQuaternion::from_euler_angles(
            std::f32::consts::FRAC_PI_2,
            0.0,
            0.0,
        ));

        Cublet {
            node: cubelet,
            front_face: front_face,
            back_face: back_face,
            left_face: left_face,
            right_face: right_face,
            top_face: top_face,
            bottom_face: bottom_face,
            position: (0, 0, 0),
        }
    }

    /// This function set the local position of the cublet (relativly to the cube)
    /// Warning: using this function will not change the real position of the cublet in the 3D space.
    /// The 'position' attribute is only used to now what cublet to move when needed...
    ///
    /// # Args
    /// * 'x, y, z': the new position of the cublet
    pub fn set_position(&mut self, x: usize, y: usize, z: usize) {
        self.position = (x, y, z);
    }
}

pub struct Gcube {
    cube: Cube,
    cublets: Vec<Cublet>,
    node: SceneNode,
}

impl Gcube {
    /// This function crate a new Gcube
    ///
    /// # Args:
    /// * '_cube' - the Cube object (to talk with solver)
    /// * 'window' - the window in which display the 3D object
    ///
    /// ## Warning:
    /// The size of the cube will depend on the size of the '_cube' parameter!
    ///
    pub fn new(_cube: Cube, _window: &mut Window) -> Self {
        let mut _node = _window.add_group();
        let s: i32 = 1;
        let mut _cublets = Vec::new();

        for i in -s..=s {
            let fx = i as f32;
            for j in -s..=s {
                let fy = j as f32;
                for k in -s..=s {
                    let fz = k as f32;
                    let mut cublet = Cublet::new();
                    let translation = Isometry3::translation(
                        fx * OFFSET + fx,
                        fy * OFFSET + fy,
                        fz * OFFSET + fz,
                    );
                    cublet.node.set_local_transformation(translation);
                    cublet.set_position((i + s) as usize, (j + s) as usize, (k + s) as usize);

                    let (x, y, z) = cublet.position;

                    if k == s {
                        let (r, g, b) = _cube.get_faces()[5][((2 - y) * 3 + (x)) as usize].to_rgb();
                        cublet.back_face.set_color(r, g, b);
                    } else if k == -s {
                        let (r, g, b) =
                            _cube.get_faces()[0][((2 - y) * 3 + (2 - x)) as usize].to_rgb();
                        cublet.front_face.set_color(r, g, b);
                    }
                    if i == s {
                        let (r, g, b) =
                            _cube.get_faces()[2][((2 - y) * 3 + (2 - z)) as usize].to_rgb();
                        cublet.left_face.set_color(r, g, b);
                    } else if i == -s {
                        let (r, g, b) = _cube.get_faces()[4][((2 - y) * 3 + (z)) as usize].to_rgb();
                        cublet.right_face.set_color(r, g, b);
                    }
                    if j == s {
                        let (r, g, b) =
                            _cube.get_faces()[1][((2 - z) * 3 + (2 - x)) as usize].to_rgb();
                        cublet.top_face.set_color(r, g, b);
                    } else if j == -s {
                        let (r, g, b) = _cube.get_faces()[3][(z * 3 + (2 - x)) as usize].to_rgb();
                        cublet.bottom_face.set_color(r, g, b);
                    }
                    let rc_child = Rc::new(RefCell::new(cublet));
                    let child_node = Rc::clone(&rc_child).borrow().node.clone();
                    let cublet = Rc::clone(&rc_child).borrow().clone();
                    _node.add_child(child_node);
                    _cublets.push(cublet);
                }
            }
        }

        Gcube {
            cube: _cube,

            cublets: _cublets,
            node: _node,
        }
    }

    pub fn set_cube(&mut self, cube: Cube) {
        let s: i32 = 1;
        let mut _cublets = Vec::new();

        for i in 0..self.cublets.len() {
            let cublet = self.cublets[i].borrow_mut();
            cublet.node.set_visible(false);
        }

        for i in -s..=s {
            let fx = i as f32;
            for j in -s..=s {
                let fy = j as f32;
                for k in -s..=s {
                    let fz = k as f32;
                    let mut cublet = Cublet::new();
                    let translation = Isometry3::translation(
                        fx * OFFSET + fx,
                        fy * OFFSET + fy,
                        fz * OFFSET + fz,
                    );
                    cublet.node.set_local_transformation(translation);
                    cublet.set_position((i + s) as usize, (j + s) as usize, (k + s) as usize);

                    let (x, y, z) = cublet.position;

                    if k == s {
                        let (r, g, b) = cube.get_faces()[5][((2 - y) * 3 + (x)) as usize].to_rgb();
                        cublet.back_face.set_color(r, g, b);
                    } else if k == -s {
                        let (r, g, b) = cube.get_faces()[0][((2 - y) * 3 + (2 - x)) as usize].to_rgb();
                        cublet.front_face.set_color(r, g, b);
                    }
                    if i == s {
                        let (r, g, b) =
                            cube.get_faces()[2][((2 - y) * 3 + (2 - z)) as usize].to_rgb();
                        cublet.left_face.set_color(r, g, b);
                    } else if i == -s {
                        let (r, g, b) = cube.get_faces()[4][((2 - y) * 3 + (z)) as usize].to_rgb();
                        cublet.right_face.set_color(r, g, b);
                    }
                    if j == s {
                        let (r, g, b) =
                            cube.get_faces()[1][((2 - z) * 3 + (2 - x)) as usize].to_rgb();
                        cublet.top_face.set_color(r, g, b);
                    } else if j == -s {
                        let (r, g, b) = cube.get_faces()[3][(z * 3 + (2 - x)) as usize].to_rgb();
                        cublet.bottom_face.set_color(r, g, b);
                    }
                    let rc_child = Rc::new(RefCell::new(cublet));
                    let child_node = Rc::clone(&rc_child).borrow().node.clone();
                    let cublet = Rc::clone(&rc_child).borrow().clone();
                    self.node.add_child(child_node);
                    _cublets.push(cublet);
                }
            }
        }

        self.cublets = _cublets;
    }

    pub fn shuffle(
        &mut self,
        window: &mut Window,
        moves: u32,
        gtk_image: &mut gtk::Image,
        camera: &mut dyn Camera,
    ) -> Vec<(Move, usize)> {

        let mut res = vec![];

        let mut rng = rand::thread_rng();

        let mut prev = 0;

        for _ in 0..moves {
            let mut r: i32 = rng.gen_range(1..=6);

            // not two times the same mv
            while (r - prev).abs() == 1 {
                r = rng.gen_range(1..=6);
            }

            prev = r;

            let mut n: usize = rng.gen_range(0..2);

            // not the middle layer
            if n == 1 {
                n = 2;
            }

            let mov = match r {
                1 => Move::H,
                2 => Move::Hp,
                3 => Move::L,
                4 => Move::Lp,
                5 => Move::V,
                _ => Move::Vp,
            };
            self.apply_move(window, mov, n, gtk_image, camera, SHUFFLE_ANIMATION_TIME);
            res.push((mov, n));
        }

        res
    }

    /// This function make the graphic cube execute move with animation
    ///
    /// # Args
    /// * 'window' - a reference to the window where the cube is displayed
    /// * 'mov' - the type of move
    /// * 'n' - the stripe concerned by the move (depending on the type)
    ///
    /// # Warning:
    /// The time of the animation will depend on the global constant 'ANIMATION_DURATION'
    pub fn apply_move(
        &mut self,
        window: &mut Window,
        mov: Move,
        n: usize,
        gtk_image: &mut gtk::Image,
        camera: &mut dyn Camera,
        duration: f32,
    ) {
        let animation_duration = Duration::from_secs_f32(duration);
        let start_time = Instant::now();
        let mut prev = 0.0;
        let s = self.cube.get_size() - 1;

        // Determine factors (the axe of the rotation)
        let (roll_fact, pitch_fact, yaw_fact) = match mov {
            Move::V => (1.0, 0.0, 0.0),
            Move::Vp => (-1.0, 0.0, 0.0),
            Move::H => (0.0, 1.0, 0.0),
            Move::Hp => (0.0, -1.0, 0.0),
            Move::L => (0.0, 0.0, 1.0),
            Move::Lp => (0.0, 0.0, -1.0),
        };

        // Functions to determine which cublet to move depending of the move type
        let f_v_move = |(x, _, _): (usize, usize, usize)| x;
        let f_h_move = |(_, y, _): (usize, usize, usize)| y;
        let f_l_move = |(_, _, z): (usize, usize, usize)| z;

        // Functions to determine which coordinates to update depending of the move type
        let f_h_coord = |cub: &mut Cublet| {
            let (x, y, z) = cub.position;
            cub.set_position(z, y, s - x);
        };
        let f_l_coord = |cub: &mut Cublet| {
            let (x, y, z) = cub.position;
            cub.set_position(s - y, x, z);
        };
        let f_v_coord = |cub: &mut Cublet| {
            let (x, y, z) = cub.position;
            cub.set_position(x, s - z, y);
        };
        let f_hp_coord = |cub: &mut Cublet| {
            let (x, y, z) = cub.position;
            cub.set_position(s - z, y, x);
        };
        let f_lp_coord = |cub: &mut Cublet| {
            let (x, y, z) = cub.position;
            cub.set_position(y, s - x, z);
        };
        let f_vp_coord = |cub: &mut Cublet| {
            let (x, y, z) = cub.position;
            cub.set_position(x, z, s - y);
        };

        let f_move = if mov == Move::H || mov == Move::Hp {
            f_h_move
        } else if mov == Move::L || mov == Move::Lp {
            f_l_move
        } else {
            f_v_move
        };

        let mut delta_sum = 0.;

        // Animation loop
        while window.render_with_camera(camera) {
            update_gtk_image(window, gtk_image);

            let elapsed_time = start_time.elapsed();
            let mut delta = (elapsed_time.as_secs_f32() - prev) / animation_duration.as_secs_f32();
            delta_sum += delta;
            prev = elapsed_time.as_secs_f32();

            if delta_sum > 1.0 {
                delta = delta - (delta_sum - 1.0);
                if delta < 0.0 {
                    delta = 0.0;
                }
            }

            let rotation_angle = delta * std::f32::consts::FRAC_PI_2;
            let rotation_quaternion = UnitQuaternion::from_euler_angles(
                rotation_angle * roll_fact,
                rotation_angle * pitch_fact,
                rotation_angle * yaw_fact,
            );

            // Apply rotation to all cublets concerned (determined by f)
            for i in 0..self.cublets.len() {
                let cublet = self.cublets[i].borrow_mut();
                if f_move(cublet.position) == n {
                    cublet.node.append_rotation(&rotation_quaternion);
                }
            }

            if elapsed_time >= animation_duration {
                break;
            }
        }

        // Update local coordinates
        for i in 0..self.cublets.len() {
            let cublet = self.cublets[i].borrow_mut();
            if f_move(cublet.position) == n {
                if mov == Move::H {
                    f_h_coord(cublet);
                } else if mov == Move::Hp {
                    f_hp_coord(cublet);
                } else if mov == Move::L {
                    f_l_coord(cublet);
                } else if mov == Move::Lp {
                    f_lp_coord(cublet);
                } else if mov == Move::V {
                    f_v_coord(cublet);
                } else {
                    f_vp_coord(cublet);
                }
            }
        }
    }
}
