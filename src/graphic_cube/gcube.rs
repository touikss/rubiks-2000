//! This module contains [`Gcube`], [`GCublet`], [`GFacelet`] definitions

// std
use std::{ borrow::BorrowMut, cell::RefCell, rc::Rc, time::{ Duration, Instant } };
use rand::Rng;

// local
use crate::{ critical, cube::{ enums::{ ColorTag, Move }, face_cube::FaceCube }, error };
use super::gtk_binding::update_gtk_image;

// others
use kiss3d::{
    camera::Camera,
    nalgebra::{ Isometry3, Point3, Quaternion, Translation3, Unit, UnitQuaternion },
    scene::SceneNode,
    window::Window,
};

pub const COMPLETE_MOVE_ANGLE: f32 = std::f32::consts::FRAC_PI_2;
pub const PREVIEW_MOVE_ANGLE: f32 = std::f32::consts::FRAC_PI_6;
const CUBLET_SIZE: f32 = 1.0;
const OFFSET: f32 = 0.2;
pub const SIMULATOR_ANIMATION_TIME: f32 = 0.4;
// pub const SOLUTION_ANIMATION_TIME: f32 = 0.8;
pub const PREVIEW_ANIMATION_TIME: f32 = 0.3;
pub const ANIMATION_ANIMATION_TIME: f32 = 0.2;
const SHUFFLE_ANIMATION_TIME: f32 = 0.1;

// index of face in the visible_face field of Cublet struct
const UP_FACE: usize = 0;
const RIGHT_FACE: usize = 1;
const FRONT_FACE: usize = 2;
const DOWN_FACE: usize = 3;
const LEFT_FACE: usize = 4;
const BACK_FACE: usize = 5;

/// A 3D cublet.
///
/// ## Fields
///
/// * `node` - its node in the **kiss3d** hierarchy
/// * `faces` - its faces node in the **kiss3d** hierarchy
/// * `position` - its position relativly to the cube
/// * `visible_faces` - the indexes of its visible faces (in `faces`)
#[derive(Clone)]
pub struct GCublet {
    /// The node in the **kiss3d** hierarchy
    pub node: SceneNode,
    /// The faces node in the **kiss3d** hierarchy
    pub faces: [SceneNode; 6],
    /// The position relativly to the cube
    /// so from 0,0,0 to 2, 2, 2
    pub position: (usize, usize, usize),
    /// The indexes of its visible faces (in `faces`)
    pub visible_faces: Vec<usize>,
}

impl GCublet {
    /// Create a new [`GCublet`].
    ///
    /// ## Returns
    ///
    /// The new [`GCublet`]
    ///
    /// ## Notes
    ///
    /// All its faces are set to black, `visible_faces` is empty and `position` is set to (0, 0, 0)
    pub fn new() -> Self {
        // create an empty node
        let mut cubelet = SceneNode::new_empty();

        // create faces (quads)
        let mut front_face = cubelet.add_quad(CUBLET_SIZE, CUBLET_SIZE, 1, 1);
        let mut back_face = cubelet.add_quad(CUBLET_SIZE, CUBLET_SIZE, 1, 1);
        let mut right_face = cubelet.add_quad(CUBLET_SIZE, CUBLET_SIZE, 1, 1);
        let mut left_face = cubelet.add_quad(CUBLET_SIZE, CUBLET_SIZE, 1, 1);
        let mut up_face = cubelet.add_quad(CUBLET_SIZE, CUBLET_SIZE, 1, 1);
        let mut down_face = cubelet.add_quad(CUBLET_SIZE, CUBLET_SIZE, 1, 1);

        // set faces positions
        let half_size = CUBLET_SIZE / 2.0;
        back_face.set_local_translation(Translation3::new(0.0, 0.0, half_size));
        front_face.set_local_translation(Translation3::new(0.0, 0.0, -half_size));
        right_face.set_local_translation(Translation3::new(-half_size, 0.0, 0.0));
        left_face.set_local_translation(Translation3::new(half_size, 0.0, 0.0));
        up_face.set_local_translation(Translation3::new(0.0, half_size, 0.0));
        down_face.set_local_translation(Translation3::new(0.0, -half_size, 0.0));

        back_face.set_local_transformation(Translation3::new(0.0, 0.0, half_size).into());
        front_face.set_local_transformation(Translation3::new(0.0, 0.0, -half_size).into());
        right_face.set_local_transformation(Translation3::new(-half_size, 0.0, 0.0).into());
        left_face.set_local_transformation(Translation3::new(half_size, 0.0, 0.0).into());
        up_face.set_local_transformation(Translation3::new(0.0, half_size, 0.0).into());
        down_face.set_local_transformation(Translation3::new(0.0, -half_size, 0.0).into());

        // set faces color to black
        back_face.set_color(0.0, 0.0, 0.0);
        front_face.set_color(0.0, 0.0, 0.0);
        left_face.set_color(0.0, 0.0, 0.0);
        right_face.set_color(0.0, 0.0, 0.0);
        up_face.set_color(0.0, 0.0, 0.0);
        down_face.set_color(0.0, 0.0, 0.0);

        // apply face rotation
        back_face.set_local_rotation(UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0));
        front_face.set_local_rotation(
            UnitQuaternion::from_euler_angles(0.0, std::f32::consts::PI, 0.0)
        );
        right_face.set_local_rotation(
            UnitQuaternion::from_euler_angles(0.0, std::f32::consts::FRAC_PI_2, 0.0)
        );
        left_face.set_local_rotation(
            UnitQuaternion::from_euler_angles(0.0, -std::f32::consts::FRAC_PI_2, 0.0)
        );
        up_face.set_local_rotation(
            UnitQuaternion::from_euler_angles(-std::f32::consts::FRAC_PI_2, 0.0, 0.0)
        );
        down_face.set_local_rotation(
            UnitQuaternion::from_euler_angles(std::f32::consts::FRAC_PI_2, 0.0, 0.0)
        );

        GCublet {
            node: cubelet,
            faces: [up_face, right_face, front_face, down_face, left_face, back_face],
            // position is not set
            position: (0, 0, 0),
            // no visible faces
            visible_faces: vec![],
        }
    }

    /// Set the local position of the cublet (relativly to the cube)
    ///
    /// # Args
    /// * `x, y, z`: the new position of the cublet
    ///
    /// ## Warning
    ///
    /// Using this function will not change the real position of the cublet in the 3D space.
    /// The 'position' attribute is only used to now what cublet to move when needed...
    pub fn set_position(&mut self, x: usize, y: usize, z: usize) {
        self.position = (x, y, z);
    }
}

/// A 3D Facelet.
///
/// ## Fields
///
/// * `node` - its node in the **kiss3d** hierarchy
/// * `parent` - its parent node in the **kiss3d** hierarchy
/// * `x_min` - its min x value in the global space
/// * `x_max` - its max x value in the global space
/// * `y_min` - its min y value in the global space
/// * `y_max` - its max y value in the global space
/// * `z_min` - its min z value in the global space
/// * `z_max` - its max z value in the global space
///
/// ## Notes
///
/// [`GFacelet`] is mainly used for the raycast. Its mins and maxs coordinates allow us to do quick collision detection.
#[derive(Clone)]
pub struct GFacelet {
    pub node: SceneNode,
    parent: SceneNode,
    gcube: SceneNode,
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
    z_min: f32,
    z_max: f32,
}

impl GFacelet {
    /// Create a new [`GFacelet`].
    ///
    /// ## Args
    ///
    /// * `parent` - its parent in the **kiss3d** hierarchy
    /// * `node` its node in the **kiss3d** hierarchy
    ///
    /// ## Returns
    ///
    /// The new [`GFacelet`]
    ///
    /// ## Notes
    ///
    /// Extremums are set.
    fn new(parent: SceneNode, node: SceneNode, gcube: SceneNode) -> Self {
        let mut facelet_node = Self {
            node,
            parent,
            gcube,
            x_min: 0.0,
            x_max: 0.0,
            y_min: 0.0,
            y_max: 0.0,
            z_min: 0.0,
            z_max: 0.0,
        };

        facelet_node.update();
        facelet_node
    }

    /// Update the extremums.
    ///
    /// ## Notes
    ///
    /// * Extremums are clamp with a precision of **0.01**.
    /// * Delta is aplyied to the coordinate where max == min.
    fn update(&mut self) {
        let mut x_min: f32 = f32::MAX;
        let mut x_max: f32 = f32::MIN;
        let mut y_min: f32 = f32::MAX;
        let mut y_max: f32 = f32::MIN;
        let mut z_min: f32 = f32::MAX;
        let mut z_max: f32 = f32::MIN;

        let gcube_transform = self.gcube.data().local_transformation();
        let parent_transform = self.parent.data().local_transformation();
        let transform = self.node.data().local_transformation();

        let mut f = |vertices: &[Point3<f32>]| {
            // go through the vertices
            for vertex in vertices {
                // update verticies transform in the cublet and in the gcube

                // vertex in the cublet
                let local_vertex = transform.transform_point(vertex.into());
                // vertex in the cube
                let gcube_vertex = parent_transform.transform_point(&local_vertex);
                // vertex in the global space
                let global_vertex = gcube_transform.transform_point(&gcube_vertex);

                // get mins and maxs
                x_min = x_min.min(global_vertex.x);
                x_max = x_max.max(global_vertex.x);

                y_min = y_min.min(global_vertex.y);
                y_max = y_max.max(global_vertex.y);

                z_min = z_min.min(global_vertex.z);
                z_max = z_max.max(global_vertex.z);
            }
        };

        self.node.read_vertices(&mut f);

        // there is a coord whose min == max for each face
        // but maybe our point is not right on it
        // so we add a delta on this particular coord to allow collision detection
        let delta = 0.05;

        // we clamp each value because float cannot be exact when moves are applied
        if (z_max * 100.0).round() / 100.0 == (z_min * 100.0).round() / 100.0 {
            z_max += delta;
            z_min -= delta;
        }
        if (y_max * 100.0).round() / 100.0 == (y_min * 100.0).round() / 100.0 {
            y_max += delta;
            y_min -= delta;
        }
        if (x_max * 100.0).round() / 100.0 == (x_min * 100.0).round() / 100.0 {
            x_max += delta;
            x_min -= delta;
        }

        self.x_min = x_min;
        self.x_max = x_max;
        self.y_min = y_min;
        self.y_max = y_max;
        self.z_min = z_min;
        self.z_max = z_max;
    }
}

#[derive(Clone)]
/// A 3D **Rubik's Cube**
///
/// ## Fields
///
/// * `node` - its node in the **kiss3d** hierarchy
/// * `face_cube` - its [`FaceCube`] representation
/// * `gcublets` - its graphical cublets
/// * `gfacelet` - its graphical **visible** facelets
pub struct GCube {
    /// The node in the **kiss3d** hierarchy
    pub node: SceneNode,
    /// The [`FaceCube`] representation
    pub face_cube: FaceCube,
    /// The graphical cublets
    pub gcublets: Vec<GCublet>,
    /// The graphical **visible** facelets
    pub gfacelets: Vec<Option<GFacelet>>,
}

impl GCube {
    /// Create a new [`GCube`]
    ///
    /// # Args:
    ///
    /// * 'face_cube' - the [`FaceCube`] object
    /// * 'window' - the **kiss3d** window in which display the 3D object
    pub fn new(face_cube: FaceCube, window: &mut Window) -> Self {
        // create the node
        let node = window.add_group();
        // cublets are empty
        let cublets = vec![];
        // facelet are empty also
        let facelets = vec![None; 54];

        let mut gcube = GCube {
            face_cube,
            gcublets: cublets,
            gfacelets: facelets,
            node,
        };

        // set the face_cube
        // will set facelets and cublets
        gcube.init_cube(face_cube);

        gcube
    }


    /// Set the `face_cube` field and configure the [`GCube`] depending on it.
    ///
    /// ## Args
    ///
    /// `face_cube` - the new [`FaceCube`]
    pub fn init_cube(&mut self, face_cube: FaceCube) {
        // the new cublets of the cube
        let mut new_cublets = Vec::new();

        // disable the old cublets
        // because kiss3D does not allow to destroy a scene node, we just set it unvisible
        for i in 0..self.gcublets.len() {
            let cublet = self.gcublets[i].borrow_mut();
            cublet.node.set_visible(false);
        }

        for i in -1..=1 {
            let fx = i as f32;
            for j in -1..=1 {
                let fy = j as f32;
                for k in -1..=1 {
                    let fz = k as f32;
                    // create the new cublet
                    let mut cublet = GCublet::new();
                    let translation = Isometry3::translation(
                        fx * OFFSET + fx,
                        fy * OFFSET + fy,
                        fz * OFFSET + fz
                    );
                    // set the global transformation of the cube
                    cublet.node.set_local_transformation(translation);

                    // set its local position
                    cublet.set_position((i + 1) as usize, (j + 1) as usize, (k + 1) as usize);

                    let (x, y, z) = cublet.position;

                    // -- BACK FACE --
                    if k == 1 {
                        // compute the index of the facelet
                        let index = 5 * 9 + (((2 - y) * 3 + x) as usize);
                        // get color from face cube
                        let (r, g, b) = face_cube.facelets()[index].to_rgb_normalized();
                        // set it to the graphical cube
                        cublet.faces[BACK_FACE].set_color(r, g, b);
                        // add it to the visible faces
                        cublet.visible_faces.push(BACK_FACE);
                        // create a facelet
                        let facelet = GFacelet::new(
                            cublet.node.clone(),
                            cublet.faces[BACK_FACE].clone(),
                            self.node.clone()
                        );
                        // add it to the corresponding index
                        self.gfacelets[index] = Some(facelet);
                    } else if
                        // -- FRONT FACE --
                        k == -1
                    {
                        let index = 2 * 9 + (2 - y) * 3 + (2 - x);
                        let (r, g, b) = face_cube.facelets()[index].to_rgb_normalized();
                        cublet.faces[FRONT_FACE].set_color(r, g, b);
                        cublet.visible_faces.push(FRONT_FACE);
                        let facelet = GFacelet::new(
                            cublet.node.clone(),
                            cublet.faces[FRONT_FACE].clone(),
                            self.node.clone()
                        );
                        self.gfacelets[index] = Some(facelet);
                    }
                    // -- LEFT FACE --
                    if i == 1 {
                        let index = 4 * 9 + (2 - y) * 3 + (2 - z);
                        let (r, g, b) = face_cube.facelets()[index].to_rgb_normalized();
                        cublet.faces[LEFT_FACE].set_color(r, g, b);
                        cublet.visible_faces.push(LEFT_FACE);
                        let facelet = GFacelet::new(
                            cublet.node.clone(),
                            cublet.faces[LEFT_FACE].clone(),
                            self.node.clone()
                        );
                        self.gfacelets[index] = Some(facelet);
                    } else if
                        // -- RIGHT FACE --
                        i == -1
                    {
                        let index = 9 + (2 - y) * 3 + z;
                        let (r, g, b) = face_cube.facelets()[index].to_rgb_normalized();
                        cublet.faces[RIGHT_FACE].set_color(r, g, b);
                        cublet.visible_faces.push(RIGHT_FACE);
                        let facelet = GFacelet::new(
                            cublet.node.clone(),
                            cublet.faces[RIGHT_FACE].clone(),
                            self.node.clone()
                        );
                        self.gfacelets[index] = Some(facelet);
                    }
                    // -- UP FACE --
                    if j == 1 {
                        let index = (2 - z) * 3 + (2 - x);
                        let (r, g, b) = face_cube.facelets()[index].to_rgb_normalized();
                        cublet.faces[UP_FACE].set_color(r, g, b);
                        cublet.visible_faces.push(UP_FACE);
                        let facelet = GFacelet::new(
                            cublet.node.clone(),
                            cublet.faces[UP_FACE].clone(),
                            self.node.clone()
                        );
                        self.gfacelets[index] = Some(facelet);
                    } else if
                        // -- DOWN FACE --
                        j == -1
                    {
                        let index = 3 * 9 + z * 3 + (2 - x);
                        let (r, g, b) = face_cube.facelets()[index].to_rgb_normalized();
                        cublet.faces[DOWN_FACE].set_color(r, g, b);
                        cublet.visible_faces.push(DOWN_FACE);
                        let facelet = GFacelet::new(
                            cublet.node.clone(),
                            cublet.faces[DOWN_FACE].clone(),
                            self.node.clone()
                        );
                        self.gfacelets[index] = Some(facelet);
                    }

                    let rc_child = Rc::new(RefCell::new(cublet));
                    let child_node = Rc::clone(&rc_child).borrow().node.clone();
                    let cublet = Rc::clone(&rc_child).borrow().clone();
                    self.node.add_child(child_node);
                    new_cublets.push(cublet);
                }
            }
        }
        self.gcublets = new_cublets;
    }

    /// Set the `face_cube` field and configure the [`GCube`] depending on it.
    ///
    /// ## Args
    ///
    /// `face_cube` - the new [`FaceCube`]
    pub fn set_face_cube(&mut self, face_cube: FaceCube) {
        // reset position of gcublets
        let mut index = 0;
        for i in -1 as i32..=1 {
            let fx = i as f32;
            for j in -1..=1 {
                let fy = j as f32;
                for k in -1..=1 {
                    let fz = k as f32;
                    let translation = Isometry3::translation(
                        fx * OFFSET + fx,
                        fy * OFFSET + fy,
                        fz * OFFSET + fz
                    );

                    let cublet = &mut self.gcublets[index];
                    // set the global transformation of the cube
                    cublet.node.set_local_transformation(translation);

                    for face in cublet.faces.iter_mut() {
                        face.set_color(0.0, 0.0, 0.0);
                    }

                    // set its local position
                    cublet.set_position((i + 1) as usize, (j + 1) as usize, (k + 1) as usize);

                    index += 1;
                }
            }
        }

        for i in 0..54 {
            let (r, g, b) = face_cube.facelets()[i].to_rgb_normalized();
            if let Some(facelet_node) = &mut self.gfacelets[i] {
                facelet_node.node.set_color(r, g, b);
                facelet_node.node.data().get_object().data().color();
            } else {
                error!("Facelet {} is not defined.", i);
            }
        }

        self.face_cube = face_cube;
        self.update_vertices();
    }

    /// Shuffle the [`GCube`] by applying random moves.
    ///
    /// ## Args
    ///
    /// * `window` - the **kiss3d** window
    /// * `moves` - how many moves to apply
    /// * `gtk_image` - the gtk image to render on
    /// * `camera` - the **kiss3d** camera
    pub fn shuffle(
        &mut self,
        window: &mut Window,
        moves: u32,
        gtk_image: &mut gtk::Image,
        camera: &mut dyn Camera
    ) {
        // init random
        let mut rng = rand::thread_rng();

        // keep the previous move
        let mut prev = 0;

        for _ in 0..moves {
            let mut r: i32 = rng.gen_range(1..=6);

            // not two times the same mv
            while (r - prev).abs() == 1 {
                r = rng.gen_range(1..=6);
            }

            prev = r;

            let mov = match r {
                1 => Move::B1,
                2 => Move::F1,
                3 => Move::L1,
                4 => Move::R1,
                5 => Move::U1,
                _ => Move::D1,
            };

            // apply the move
            self.apply_move(
                window,
                mov,
                gtk_image,
                camera,
                SHUFFLE_ANIMATION_TIME,
                COMPLETE_MOVE_ANGLE
            );
        }
    }

    ///  Execute a move with animation.
    ///
    /// # Args
    /// * 'window' - the **kiss3d** window where the cube is displayed
    /// * 'mov' - the type of move
    /// * `gtk_image` - the gtk image to render on
    /// * `camera` - the ** kiss3d** camera
    /// * `duration` - the duration of the animation
    ///
    /// ## Notes
    /// Update the [`GFacelet`] vertices after the move.
    pub fn apply_move(
        &mut self,
        window: &mut Window,
        mov: Move,
        gtk_image: &mut gtk::Image,
        camera: &mut dyn Camera,
        duration: f32,
        angle: f32
    ) {
        let animation_duration = Duration::from_secs_f32(duration);
        let start_time = Instant::now();
        let mut prev = 0.0;

        // determine factors (the axe of the rotation)
        let (roll_fact, pitch_fact, yaw_fact) = match mov {
            Move::L1 | Move::R3 => (-1.0, 0.0, 0.0),
            Move::U1 | Move::D3 => (0.0, -1.0, 0.0),
            Move::B1 | Move::F3 => (0.0, 0.0, -1.0),
            Move::R1 | Move::L3 => (1.0, 0.0, 0.0),
            Move::D1 | Move::U3 => (0.0, 1.0, 0.0),
            Move::F1 | Move::B3 => (0.0, 0.0, 1.0),
            _ =>
                critical!("Graphical cube can only move with L1, R1, U1, D1, F1, B1, L3, R3, U3, D3, F3, B3. Attempt to move : {}", mov),
        };

        //fdunctions to determine which cublet to move depending of the move type
        // h = horizontal = up or down
        // v = vertical = left or right
        // l = lateral = back or front
        let f_v_move: fn((usize, usize, usize)) -> usize = |(x, _, _): (usize, usize, usize)| x;
        let f_h_move: fn((usize, usize, usize)) -> usize = |(_, y, _): (usize, usize, usize)| y;
        let f_l_move: fn((usize, usize, usize)) -> usize = |(_, _, z): (usize, usize, usize)| z;

        // functions to determine which coordinates to update depending of the move type
        let f_d_coord = |cublet: &mut GCublet| {
            let (x, y, z) = cublet.position;
            cublet.set_position(z, y, 2 - x);
        };
        let f_f_coord = |cublet: &mut GCublet| {
            let (x, y, z) = cublet.position;
            cublet.set_position(2 - y, x, z);
        };
        let f_r_coord = |cublet: &mut GCublet| {
            let (x, y, z) = cublet.position;
            cublet.set_position(x, 2 - z, y);
        };
        let f_u_coord = |cublet: &mut GCublet| {
            let (x, y, z) = cublet.position;
            cublet.set_position(2 - z, y, x);
        };
        let f_b_coord = |cublet: &mut GCublet| {
            let (x, y, z) = cublet.position;
            cublet.set_position(y, 2 - x, z);
        };
        let f_l_coord = |cublet: &mut GCublet| {
            let (x, y, z) = cublet.position;
            cublet.set_position(x, z, 2 - y);
        };

        let (f_move, n) = match mov {
            Move::U1 | Move::U3 => (f_h_move, 2),
            Move::D1 | Move::D3 => (f_h_move, 0),
            Move::F1 | Move::F3 => (f_l_move, 0),
            Move::B1 | Move::B3 => (f_l_move, 2),
            Move::L1 | Move::L3 => (f_v_move, 2),
            Move::R1 | Move::R3 => (f_v_move, 0),
            _ =>
                critical!("Graphical cube can only move with L1, R1, U1, D1, F1, B1, L3, R3, U3, D3, F3, B3. Attempt to move : {}", mov),
        };

        let mut delta_sum = 0.0;

        // dnimation loop
        while window.render_with_camera(camera) {
            // update the gtk image
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

            let rotation_angle = delta * angle;
            let rotation_quaternion = UnitQuaternion::from_euler_angles(
                rotation_angle * roll_fact,
                rotation_angle * pitch_fact,
                rotation_angle * yaw_fact
            );

            // Apply rotation to all cublets concerned (determined by f)
            for i in 0..self.gcublets.len() {
                let cublet = self.gcublets[i].borrow_mut();
                if f_move(cublet.position) == n {
                    cublet.node.append_rotation(&rotation_quaternion);
                }
            }

            while gtk::events_pending() {
                gtk::main_iteration();
            }

            if elapsed_time > animation_duration {
                break;
            }
        }

        // Update local coordinates
        for i in 0..self.gcublets.len() {
            let cublet = self.gcublets[i].borrow_mut();
            if f_move(cublet.position) == n {
                match mov {
                    Move::U1 | Move::D3 => f_u_coord(cublet),
                    Move::D1 | Move::U3 => f_d_coord(cublet),
                    Move::F1 | Move::B3 => f_f_coord(cublet),
                    Move::B1 | Move::F3 => f_b_coord(cublet),
                    Move::L1 | Move::R3 => f_l_coord(cublet),
                    Move::R1 | Move::L3 => f_r_coord(cublet),
                    // will never happend because tested previously
                    _ => critical!("Should never happend."),
                }
            }
        }

        // update each face vertices
        self.update_vertices();

        // apply the move to the face cube
        let mut cubie_cube = self.face_cube.to_cubie_cube();
        cubie_cube.multiply(mov.get_cube());

        self.face_cube = cubie_cube.to_facelet_cube();
    }

    /// Find the [`GFacelet`] colliding with a point if it exists.
    ///
    /// ## Args
    ///
    /// * `x,y,z` - the coordinates of the point
    ///
    /// ## Returns
    ///
    /// * The index of the [GFacelet] collided in `facelets` if it exists
    /// * [None] otherwise
    pub fn find_facelet_colliding(&mut self, x: f32, y: f32, z: f32) -> Option<usize> {
        for (index, facelet_opt) in &mut self.gfacelets.iter_mut().enumerate() {
            if let Some(facelet) = &facelet_opt {
                // check for collision
                if
                    z <= facelet.z_max &&
                    z >= facelet.z_min &&
                    y <= facelet.y_max &&
                    y >= facelet.y_min &&
                    x <= facelet.x_max &&
                    x >= facelet.x_min
                {
                    return Some(index);
                }
            } else {
                critical!("Cube not defined at index : {}", index);
            }
        }
        None
    }

    /// Update all the vertices of each cublet and facelets of the cube.
    ///
    /// ## Notes
    ///
    /// This is primordial for the raycasting part.
    pub fn update_vertices(&mut self) {
        // update each face vertices
        for facelet_opt in &mut self.gfacelets {
            if let Some(facelet) = facelet_opt {
                facelet.update();
            }
        }
    }

    /// Append a rotation to the cube and update the vertices.
    ///
    /// ## Args
    ///
    /// * `rotation` - the rotation to append
    pub fn append_rotation(&mut self, rotation: &Unit<Quaternion<f32>>) {
        // append rotation
        self.node.append_rotation(rotation);
        // update the vertices
        self.update_vertices()
    }

    pub fn highlight_slice(&mut self, slice: ColorTag) {
        let f_v_move: fn((usize, usize, usize)) -> usize = |(x, _, _): (usize, usize, usize)| x;
        let f_h_move: fn((usize, usize, usize)) -> usize = |(_, y, _): (usize, usize, usize)| y;
        let f_l_move: fn((usize, usize, usize)) -> usize = |(_, _, z): (usize, usize, usize)| z;

        let (f_slice, n) = match slice {
            ColorTag::U => (f_h_move, 2),
            ColorTag::D => (f_h_move, 0),
            ColorTag::F => (f_l_move, 0),
            ColorTag::B => (f_l_move, 2),
            ColorTag::L => (f_v_move, 2),
            ColorTag::R => (f_v_move, 0),
            _ => critical!("Invalid slice to highlight."),
        };

        let mut highlited: Vec<usize> = ((slice as usize) * 9..((slice as usize) + 1) *
            9).collect();

        match slice {
            ColorTag::U => {
                highlited.extend(vec![36, 37, 38, 18, 19, 20, 9, 10, 11, 45, 46, 47]);
            }
            ColorTag::D => {
                highlited.extend(vec![42, 43, 44, 24, 25, 26, 15, 16, 17, 51, 52, 53]);
            }
            ColorTag::F => {
                highlited.extend(vec![6, 7, 8, 9, 12, 15, 27, 28, 29, 38, 41, 44]);
            }
            ColorTag::B => {
                highlited.extend(vec![0, 1, 2, 11, 14, 17, 36, 39, 42, 33, 34, 35]);
            }
            ColorTag::L => {
                highlited.extend(vec![0, 3, 6, 18, 21, 24, 27, 30, 33, 47, 50, 53]);
            }
            ColorTag::R => {
                highlited.extend(vec![2, 5, 8, 20, 23, 26, 29, 32, 35, 45, 48, 51]);
            }
            _ => critical!("Invalid slice to highlight."),
        }

        // Apply rotation to all cublets concerned (determined by f)
        for i in 0..self.gcublets.len() {
            let cublet = self.gcublets[i].borrow_mut();
            if f_slice(cublet.position) == n {
                cublet.node.set_color(0.0, 1.0, 1.0);
            }
        }
    }
}
