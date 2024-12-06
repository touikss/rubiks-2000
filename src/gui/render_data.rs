//! This module contains the [`RenderData`] definitions

use std::{ f32::consts::PI, time::{ Duration, Instant } };

use crate::{
    critical,
    cube::{ defs::*, enums::{ ColorTag, Move } },
    graphic_cube::gcube::{
        GCube,
        ANIMATION_ANIMATION_TIME,
        COMPLETE_MOVE_ANGLE,
        PREVIEW_ANIMATION_TIME,
        PREVIEW_MOVE_ANGLE,
        SIMULATOR_ANIMATION_TIME,
        //SOLUTION_ANIMATION_TIME,
    },
    Parameters,
};

use super::{ utils::*, widget_pool::WidgetPool };

use gtk::prelude::*;

use kiss3d::{
    camera::{ Camera, FirstPerson },
    light::Light,
    nalgebra::{ Point2, Translation3, UnitQuaternion, Vector2 },
    ncollide3d::math::Point,
    window::Window as KissWindow,
};

#[derive(PartialEq, Eq)]
pub enum MoveType {
    Preview,
    Simulator,
    Solution,
    Animation,
}

/// Store rendering datas.
///
/// ## Fields
///
/// * `window` - the **kiss3d** window
/// * `camera` - the **kiss3d** camera
/// * `gtk_image` - the gtk image where **kiss3d** view will be rendered
/// * `gcube` - the graphicale object to interact with in the **kiss3d** view
///
/// ## Notes
/// [`RenderData`] is used in simulator.rs and solution.rs (the 2 view where we use 3D rendering)
pub struct RenderData {
    /// The `kiss3d` window used to render.
    ///
    /// ## Notes
    ///
    /// This window is hidden.
    pub(super) window: KissWindow,
    /// The `kiss3d` camera.
    pub(super) camera: FirstPerson,
    /// The gtk image where **kiss3d** view will be rendered
    pub(super) gtk_image: gtk::Image,
    /// The graphicale object to interact with in the **kiss3d** view
    pub(super) gcube: GCube,
}

impl RenderData {
    /// Create a new [`RenderData`].
    ///
    /// ## Args
    ///
    /// * `widget_pool` - the gtk widget pool
    /// * `render_image_name` - the name of the image where to render
    ///
    /// ## Returns
    ///
    /// The new [`RenderData`].
    ///
    /// ## Note
    ///
    /// Will create a default `camera` and `gcube`.
    pub fn new(widget_pool: &WidgetPool, render_image_name: &str) -> Self {
        // create new hidden kiss 3D window
        let mut window = kiss3d::window::Window::new_hidden("render_image_name");

        window.set_framerate_limit(Some(FRAME_LIMIT));

        // set light position
        window.set_light(Light::Absolute(Point::new(3.0, 3.0, 3.0)));

        // get main_window
        let main_window: gtk::Window = widget_pool.get(SIM_WINDOW);
        // get style context
        let style_context = main_window.style_context();
        // get background color
        if let Some(color) = style_context.lookup_color("theme_bg_color") {
            // if it exists, set 3D backround to the same color
            // thus, 3d intregration is prettier (plus jolis quoi - je sais pas parler anglais)
            window.set_background_color(
                color.red() as f32,
                color.green() as f32,
                color.blue() as f32
            );
        } else {
            // default is black
            window.set_background_color(0.0, 0.0, 0.0);
        }

        // set basic orientation and pos of camera
        // it is locked toward the cube
        let camera = FirstPerson::new(
            Point::new(CAMERA_BASE.0, CAMERA_BASE.1, CAMERA_BASE.2),
            Point::origin()
        );

        // the cube used by the graphical cube
        // each time a move is applied to the gcube, it is applied to the normal cube
        // thus, we just have to send the cube to the solver when needed
        let cube = DEFAULT.to_facelet_cube();

        // the graphical cube
        let gcube = GCube::new(cube, &mut window);

        // the gtk rendering image
        let gtk_image: gtk::Image = widget_pool.get(render_image_name);

        Self {
            window,
            camera,
            gtk_image,
            gcube,
        }
    }

    /// Render on the `gtk_image`.
    ///
    /// ## Panics
    ///
    /// Rendering failed.
    pub fn render(&mut self) {
        // try render
        if self.window.render_with_camera(&mut self.camera) {
            // add delay to be sure render is finished
            delay(FRAME_INTERVAL);
            // update gtk image
            self.update_gtk_image_rd();
        } else {
            critical!("Failed rendering.");
        }
    }

    /// Render a move animation.
    ///
    /// ## Args
    ///
    /// `mov` - the move to render
    ///
    /// ## Panics
    ///
    /// Rendering failed.
    pub fn render_move(&mut self, mov: Move, move_type: MoveType) {
        // it is a move 2 so just apply to times its move 1

        let (angle, duration) = match move_type {
            MoveType::Preview => (PREVIEW_MOVE_ANGLE, PREVIEW_ANIMATION_TIME),
            MoveType::Simulator => (COMPLETE_MOVE_ANGLE, SIMULATOR_ANIMATION_TIME),
            MoveType::Solution => (COMPLETE_MOVE_ANGLE, Parameters::solution_animation_duration()),
            MoveType::Animation => (COMPLETE_MOVE_ANGLE, ANIMATION_ANIMATION_TIME),
        };

        if (mov as usize) % 3 == 1 {
            let move_1 = Move::from((mov as usize) - 1);

            self.gcube.apply_move(
                &mut self.window,
                move_1,
                &mut self.gtk_image,
                &mut self.camera,
                duration,
                angle
            );
            self.gcube.apply_move(
                &mut self.window,
                move_1,
                &mut self.gtk_image,
                &mut self.camera,
                duration,
                angle
            );
        } else {
            self.gcube.apply_move(
                &mut self.window,
                mov,
                &mut self.gtk_image,
                &mut self.camera,
                duration,
                angle
            );
        }

        // render last time to be sure final animation rendering is finished
        self.render();
    }

    /// Render a cube rotation
    ///
    /// ## Args
    ///
    /// * `yaw` - the rotation around the y axis
    /// * `pitch` - the rotation around the x axis
    /// * `roll` - the rotation around the z axis
    ///
    /// ## Notes
    ///
    /// * The rotation will be added to the cube not just set.
    /// * Be aware that it is the `gcube` which will be rotated, not the `camera`.
    pub fn render_cube_rotation(&mut self, yaw: f32, pitch: f32, roll: f32, duration: f32) {
        let animation_duration = Duration::from_secs_f32(duration);
        let start_time = Instant::now();

        let mut prev = 0.0;
        let mut elapsed_time = start_time.elapsed();

        // delta sum is used to set the exact rotation asked at the end of the animation
        let mut delta_sum = 0.0;

        while elapsed_time <= animation_duration {
            elapsed_time = start_time.elapsed();

            // compute the time ellapsed since last loop tour
            let mut delta = (elapsed_time.as_secs_f32() - prev) / animation_duration.as_secs_f32();
            prev = elapsed_time.as_secs_f32();

            delta_sum += delta;

            if delta_sum > 1.0 {
                delta = delta - (delta_sum - 1.0);
                if delta < 0.0 {
                    delta = 0.0;
                }
            }

            // compute delta rotation
            let delta_rotation = UnitQuaternion::from_euler_angles(
                delta * roll,
                delta * pitch,
                delta * yaw
            );

            // append it
            self.gcube.node.append_rotation(&delta_rotation);

            // render
            self.render();

            // ....
            gtk::main_iteration();
            gtk::main_iteration();
            gtk::main_iteration();
        }
        self.render();

        // update all the vertices of the cube after the rotation
        self.gcube.update_vertices();
    }

    pub fn render_animation(&mut self) {
        let dist = 40.0;

        // Calculate the direction vector
        let forward = self.camera.eye_dir();

        let translation = Translation3::from(forward * -dist);
        //self.camera.look_at(&translation);
        self.camera.translate_mut(&translation);

        let animation_duration = Duration::from_secs_f32(ROTATION_DURATION_ANIMATION);
        let start_time = Instant::now();

        let mut prev = 0.0;
        let mut elapsed_time = start_time.elapsed();

        // delta sum is used to set the exact rotation asked at the end of the animation
        let mut delta_sum = 0.0;

        while elapsed_time <= animation_duration {
            elapsed_time = start_time.elapsed();

            // compute the time ellapsed since last loop tour
            let mut delta = (elapsed_time.as_secs_f32() - prev) / animation_duration.as_secs_f32();
            prev = elapsed_time.as_secs_f32();

            delta_sum += delta;

            if delta_sum > 1.0 {
                delta = delta - (delta_sum - 1.0);
                if delta < 0.0 {
                    delta = 0.0;
                }
            }

            // compute delta rotation
            let delta_rotation = UnitQuaternion::from_euler_angles(0.0, delta * 4.0 * PI, 0.0);

            // append it
            self.gcube.node.append_rotation(&delta_rotation);
            let translation = Translation3::from(forward * delta * dist);
            //self.camera.look_at(&translation);
            self.camera.translate_mut(&translation);

            // render
            self.render();

            // ....
            gtk::main_iteration();
            gtk::main_iteration();
            gtk::main_iteration();
        }
        self.render();

        // update all the vertices of the cube after the rotation
        self.gcube.update_vertices();
    }

    /// Render a shuffle animation.
    ///
    /// ## Panics
    ///
    /// Rendering failed.
    pub fn render_shuffle(&mut self) {
        self.gcube.shuffle(
            &mut self.window,
            Parameters::shuffle_length(),
            &mut self.gtk_image,
            &mut self.camera
        );
        // render last time to be sure final animation rendering is finished
        self.render();
    }

    /// Draw the **kiss3d** view on the `gtk_image`.
    ///
    /// ## Notes
    ///
    /// This is the main function which handle binding between **kiss3D**
    /// and the `gtk_image` where we want to draw our 3D renderer.lay(FRAME_INTERVAL);
    /// update gtk image
    /// It has to be called in every loop you call window.render() ( so every frame basically)
    pub fn update_gtk_image_rd(&mut self) {
        // snap the window renderer (as taking a screenshor)
        let snaped = self.window.snap_image().into_vec();

        // create bytes from it
        let bytes = gtk::glib::Bytes::from(&snaped);

        // then create a pixel buffer
        let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_bytes(
            &bytes,
            gtk::gdk_pixbuf::Colorspace::Rgb,
            false,
            8,
            self.window.width() as i32,
            self.window.height() as i32,
            (self.window.width() as i32) * 3
        );

        // this code is not needed
        // treat gtk events to avoid unresponding ui
        //while gtk::events_pending() {
        //    gtk::main_iteration();
        //}

        // update the gtk image with the created pixel buffer
        self.gtk_image.set_from_pixbuf(Some(&pixbuf));
    }

    /// Send a raycast at a certain point on the kiss3D render and return the first facelet it collide.
    ///
    /// ## Args
    ///
    /// * `x` - the abscisse of the point
    /// * `y` - the ordinates of the point
    /// * `window_width` - the width of the window
    /// * `window_height` - the height of the window
    ///
    /// ## Returns
    ///
    /// * the index in the `gcube` of the facelet pointed if it exists
    /// * [`None`] otherwise
    pub fn raycast(
        &mut self,
        x: f32,
        y: f32,
        window_width: f32,
        window_height: f32
    ) -> Option<usize> {
        // convert it in kiss3D cursor coordinates depending on the image size
        let cursor = Point2::new(x, y);

        // the size of the gtk image where is rendered kiss 3D
        let size = Vector2::new(window_width, window_height);

        // project the cursor point into the 3D space to get a direction
        // point = camera pos almost
        let (point, dir) = self.camera.unproject(&cursor, &size);

        // how many point to test for ray casting
        let point_factor = 10 as f32;

        for i in 4 * (point_factor as i32)..10 * (point_factor as i32) {
            // create point on the line pointed
            let x = point.x + ((i as f32) * dir.x) / point_factor;
            let y = point.y + ((i as f32) * dir.y) / point_factor;
            let z = point.z + ((i as f32) * dir.z) / point_factor;

            // check collision
            if let Some(facelet) = self.gcube.find_facelet_colliding(x, y, z) {
                return Some(facelet);
            }
        }
        None
    }

    pub fn render_preview_move(&mut self, mv: Move) {
        //self.highlight_cube_slice(mv);

        if Parameters::show_preview() {
            self.render_move(mv, MoveType::Preview);
            self.render_move(mv.inverse(), MoveType::Preview);

            self.render();
        }
    }

    pub fn highlight_cube_slice(&mut self, mv: Move) {
        let slice = match mv {
            Move::U1 | Move::U3 | Move::U2 => ColorTag::U,
            Move::D1 | Move::D3 | Move::D2 => ColorTag::D,
            Move::F1 | Move::F3 | Move::F2 => ColorTag::F,
            Move::B1 | Move::B3 | Move::B2 => ColorTag::B,
            Move::L1 | Move::L3 | Move::L2 => ColorTag::L,
            Move::R1 | Move::R3 | Move::R2 => ColorTag::R,
            _ => critical!("Invalid move to preview."),
        };

        self.gcube.highlight_slice(slice);
        self.render();
    }

    pub fn reset_cube_color(&mut self) {
        self.gcube.set_face_cube(self.gcube.face_cube);

        self.render();
    }
}
