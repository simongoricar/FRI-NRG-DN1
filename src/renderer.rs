use std::time::Instant;

use chrono::{DateTime, Local, Utc};
use image::{ImageFormat, RgbaImage};
use miette::Result;
use nalgebra::{Isometry3, Matrix4, Orthographic3, Perspective3, Point3, Vector3, Vector4};
use parking_lot::RwLock;
use tracing::{debug, error, info};
use winit::{
    event::{ElementState, MouseButton, WindowEvent},
    keyboard::{Key, NamedKey},
};

use crate::{
    configuration::Configuration,
    splat_decoder::{Splat, Splats},
};

pub trait PixelSurfaceRenderer {
    fn draw(&self, frame: &mut [u8]);
    fn handle_window_event(&mut self, window_evnt: &WindowEvent) -> Result<()>;
}



#[inline]
fn get_pixel_coordinates_from_projected_coordinates(
    projected_position: Vector4<f32>,
    render_width: u32,
    render_height: u32,
) -> Option<(u32, u32)> {
    // debug!("Before processing: {:?}", projected_position);

    let mut projected_x = *projected_position.get(0).unwrap();
    let mut projected_y = *projected_position.get(1).unwrap();
    let mut projected_z = *projected_position.get(2).unwrap();
    let projected_w = *projected_position.get(3).unwrap();

    projected_x /= 1f32 / projected_w;
    projected_y /= 1f32 / projected_w;
    projected_z /= 1f32 / projected_w;

    // debug!(
    //     "After /= with w: {},{},{},{}",
    //     projected_x, projected_y, projected_z, projected_w
    // );

    projected_x /= 1f32 / projected_z;
    projected_y /= 1f32 / projected_z;


    if !(-1.0..=1.0).contains(&projected_x) {
        return None;
    }

    if !(-1.0..=1.0).contains(&projected_y) {
        return None;
    }


    // x and y are now guaranteed to be between -1 and 1,
    // so the next step is to remap them into u32 render coordinates.
    let render_x: u32 = (((projected_x + 1.0) / 2.0) * (render_width as f32 - 1.0)).round() as u32;
    let render_y: u32 = (((projected_y + 1.0) / 2.0) * (render_height as f32 - 1.0)).round() as u32;


    #[cfg(debug_assertions)]
    {
        if render_x >= render_width {
            panic!(
                "render_x is larger than render width: {}",
                render_x
            );
        }

        if render_y >= render_height {
            panic!(
                "render_y is larger than render height: {}",
                render_y
            );
        }
    }

    Some((render_x, render_y))
}

fn get_average_splat_coordinates(splats: &[Splat]) -> Point3<f32> {
    let average_splat_position: Point3<f32> = {
        let mut total_position = Point3::new(0f32, 0f32, 0f32);

        for splat in splats {
            for (index, value) in total_position.iter_mut().enumerate() {
                if index == 0 {
                    *value += *splat.position.get(0).unwrap();
                } else if index == 1 {
                    *value += *splat.position.get(1).unwrap();
                } else if index == 2 {
                    *value += *splat.position.get(2).unwrap();
                } else {
                    panic!("BUG: Point3 should only have three components.");
                }
            }
        }

        total_position / (splats.len() as f32)
    };

    average_splat_position
}


struct SplatRendererInner {
    pending_rerender: bool,

    camera_position: Point3<f32>,

    camera_look_target: Point3<f32>,

    forward_vector: Vector3<f32>,

    side_vector: Vector3<f32>,

    up_vector: Vector3<f32>,

    /// RGBA (u8 each) for each pixel.
    frame: Vec<u8>,
}

struct SplatRendererUserControlState {
    left_mouse_pressed: bool,
    control_key_pressed: bool,
}

pub struct SplatRenderer {
    configuration: Configuration,

    render_width: u32,

    render_height: u32,

    splat_file: Splats,

    user_control: SplatRendererUserControlState,

    inner: RwLock<SplatRendererInner>,
}

impl SplatRenderer {
    pub fn new(
        configuration: Configuration,
        render_width: u32,
        render_height: u32,
        splat_file: Splats,
        initial_camera_position: Option<Point3<f32>>,
        initial_camera_look_target: Option<Point3<f32>>,
        initial_camera_up_vector: Option<Vector3<f32>>,
    ) -> Self {
        let camera_position = initial_camera_position.unwrap_or_else(|| Point3::new(3.0, 3.0, 3.0));
        debug!("Starting camera position: {:?}", camera_position);

        let camera_look_target = initial_camera_look_target
            .unwrap_or_else(|| get_average_splat_coordinates(&splat_file.splats));
        debug!(
            "Starting camera look target: {:?}",
            camera_look_target
        );


        let initial_up_vector = initial_camera_up_vector
            .unwrap_or_else(|| Vector3::new(0.0, 1.0, 0.0))
            .normalize();
        debug!(
            "Starting camera up vector: {:?}",
            initial_up_vector
        );


        let forward_vector = (camera_look_target - camera_position).normalize();
        let side_vector = forward_vector.cross(&initial_up_vector).normalize();
        let up_vector = side_vector.cross(&forward_vector).normalize();


        let frame = vec![0; render_width as usize * render_height as usize * 4];

        let user_control = SplatRendererUserControlState {
            left_mouse_pressed: false,
            control_key_pressed: false,
        };

        let inner = RwLock::new(SplatRendererInner {
            pending_rerender: true,
            camera_position,
            camera_look_target,
            forward_vector,
            side_vector,
            up_vector,
            frame,
        });


        Self {
            configuration,
            render_width,
            render_height,
            splat_file,
            user_control,
            inner,
        }
    }

    pub fn render_in_place(&self) {
        info!("Rendering splat.");

        let mut inner_locked = self.inner.write();

        // Transform the world coordinates of each splat to camera coordinates.

        let updated_forward_vector =
            (inner_locked.camera_look_target - inner_locked.camera_position).normalize();
        let updated_side_vector = updated_forward_vector
            .cross(&inner_locked.up_vector)
            .normalize();
        let updated_up_vector = updated_side_vector
            .cross(&updated_forward_vector)
            .normalize();

        inner_locked.forward_vector = updated_forward_vector;
        inner_locked.side_vector = updated_side_vector;
        inner_locked.up_vector = updated_up_vector;


        // let look_at_matrix = Matrix4::<f32>::look_at_lh(
        //     &inner_locked.camera_position,
        //     &inner_locked.camera_look_target,
        //     &updated_up_vector,
        // );

        debug!(
            "Camera position: {:?}\nCamera look target: {:?}\n\
            Forward vector: {:?}\nSide vector: {:?}\nUp vector: {:?}",
            inner_locked.camera_position,
            inner_locked.camera_look_target,
            updated_forward_vector,
            updated_side_vector,
            updated_up_vector
        );

        let look_at_matrix = Isometry3::look_at_rh(
            &inner_locked.camera_position,
            &inner_locked.camera_look_target,
            &updated_up_vector,
        );

        let projection_matrix = Perspective3::<f32>::new(
            self.render_width as f32 / self.render_height as f32,
            // 0.6,
            45f32,
            0.1,
            100.0,
        );


        debug!(
            "Look at matrix:\n{:?}",
            look_at_matrix.to_matrix()
        );
        debug!(
            "Projection matrix:\n{:?}",
            projection_matrix.as_matrix()
        );

        // let projection_matrix = Orthographic3::new(10.0, 1.0, 20.0, 2.0, 1000.0, 0.1);


        let joint_matrix = projection_matrix.as_matrix() * look_at_matrix.to_matrix();


        // Reset canvas.
        for some_pixel_component in inner_locked.frame.iter_mut() {
            *some_pixel_component = 0;
        }

        // Draw to canvas.
        for splat in self.splat_file.splats.iter() {
            let position_in_world_space = Vector4::new(
                splat.position.x,
                splat.position.y,
                splat.position.z,
                1f32,
            );

            // let position_in_camera_space = look_at_matrix * position_in_world_space;
            // let position_in_clip_space = projection_matrix.as_matrix() * position_in_camera_space;

            println!(
                "Projecting splat at position {:?}",
                splat.position
            );

            let position_in_clip_space = joint_matrix * position_in_world_space;

            if let Some((render_x, render_y)) = get_pixel_coordinates_from_projected_coordinates(
                position_in_clip_space,
                self.render_width,
                self.render_height,
            ) {
                let pixel_index = ((render_y * self.render_width + render_x) * 4) as usize;

                println!(
                    "Splat is in the viewport at {}x{}, which is index {}.",
                    render_x, render_y, pixel_index
                );

                inner_locked.frame.as_mut_slice()[pixel_index..pixel_index + 4]
                    .copy_from_slice(splat.color.as_slice());
            } else {
                println!("Splat is not in the viewport.");
            }
        }

        inner_locked.pending_rerender = false;
    }

    fn save_screenshot_to_disk(&self) {
        let screenshot_time_string = Local::now().format("%Y-%m-%d_%H-%M-%S-%3f");
        let screenshot_name = format!("nrg-screenshot_{}.png", screenshot_time_string);

        let full_screenshot_path = self
            .configuration
            .screenshot
            .screenshot_directory_path
            .join(&screenshot_name);


        let buffer_as_image = {
            let inner_locked = self.inner.read();


            let opaque_frame = {
                let mut cloned_frame = inner_locked.frame.clone();

                for pixel in cloned_frame.chunks_exact_mut(4) {
                    pixel[3] = 255;
                }

                cloned_frame
            };

            let Some(image) = RgbaImage::from_vec(
                self.render_width,
                self.render_height,
                opaque_frame,
            ) else {
                error!("Failed to save screenshot: buffer is not big enough.");
                return;
            };

            image
        };


        let save_result = buffer_as_image.save_with_format(full_screenshot_path, ImageFormat::Png);

        if let Err(save_error) = save_result {
            error!(
                "Failed to asve screenshot: erorred while saving as PNG: {:?}",
                save_error
            );
        }

        info!("Screenshot save to disk as {}.", screenshot_name);
    }
}

impl PixelSurfaceRenderer for SplatRenderer {
    fn draw(&self, frame: &mut [u8]) {
        let inner_locked_read_only = self.inner.read();

        if inner_locked_read_only.pending_rerender {
            drop(inner_locked_read_only);

            debug!("Resolving pending rerender.");
            let time_render_start = Instant::now();
            self.render_in_place();
            debug!(
                "Rerender took {} milliseconds.",
                (time_render_start.elapsed().as_secs_f64() * 1000.0).round() as u32
            );

            let inner_locked_read_only = self.inner.read();
            frame.copy_from_slice(&inner_locked_read_only.frame);
        } else {
            frame.copy_from_slice(&inner_locked_read_only.frame);
        }

        // for (pixel_index, pixel) in frame.chunks_exact_mut(4).enumerate() {
        //      pixel.copy_from_slice(&[133, 255, 211, 255]);
        // }
    }

    fn handle_window_event(&mut self, window_event: &WindowEvent) -> Result<()> {
        const MOVE_CAMERA_BY: f32 = 0.1;

        match window_event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let Key::Named(named_key) = &event.logical_key {
                    if named_key == &NamedKey::Control {
                        match event.state {
                            ElementState::Pressed => {
                                info!("User is holding down Ctrl key.");
                                self.user_control.control_key_pressed = true;
                            }
                            ElementState::Released => {
                                info!("User released Ctrl key.");
                                self.user_control.control_key_pressed = false;
                            }
                        }
                    }
                };


                let Key::Character(input_key) = &event.logical_key else {
                    return Ok(());
                };

                if event.state != ElementState::Released {
                    return Ok(());
                }


                // Check for Ctrl+S (screenshot shortcut).
                if input_key == "s" && self.user_control.control_key_pressed {
                    info!("User pressed \"Ctrl+s\", saving screenhot.");
                    self.save_screenshot_to_disk();
                    return Ok(());
                }


                if !self.user_control.left_mouse_pressed && self.user_control.control_key_pressed {
                    return Ok(());
                }


                let mut inner_locked = self.inner.write();


                if input_key == "s" {
                    info!(
                        "User pressed \"s\", moving camera x backwards by {}.",
                        MOVE_CAMERA_BY
                    );

                    inner_locked.camera_position.x -= MOVE_CAMERA_BY;
                    // inner_locked.camera_look_target.x -= MOVE_CAMERA_BY;

                    inner_locked.pending_rerender = true;
                } else if input_key == "w" {
                    info!(
                        "User pressed \"w\", moving camera x forwards by {}.",
                        MOVE_CAMERA_BY
                    );

                    inner_locked.camera_position.x += MOVE_CAMERA_BY;
                    // inner_locked.camera_look_target.x += MOVE_CAMERA_BY;

                    inner_locked.pending_rerender = true;
                } else if input_key == "d" {
                    info!(
                        "User pressed \"d\", moving camera y backwards by {}.",
                        MOVE_CAMERA_BY
                    );

                    inner_locked.camera_position.y -= MOVE_CAMERA_BY;
                    // inner_locked.camera_look_target.y -= MOVE_CAMERA_BY;

                    inner_locked.pending_rerender = true;
                } else if input_key == "e" {
                    info!(
                        "User pressed \"e\", moving camera y forwards by {}.",
                        MOVE_CAMERA_BY
                    );

                    inner_locked.camera_position.y += MOVE_CAMERA_BY;
                    // inner_locked.camera_look_target.y += MOVE_CAMERA_BY;

                    inner_locked.pending_rerender = true;
                } else if input_key == "f" {
                    info!(
                        "User pressed \"f\", moving camera z backwards by {}.",
                        MOVE_CAMERA_BY
                    );

                    inner_locked.camera_position.z -= MOVE_CAMERA_BY;
                    // inner_locked.camera_look_target.z -= MOVE_CAMERA_BY;

                    inner_locked.pending_rerender = true;
                } else if input_key == "r" {
                    info!(
                        "User pressed \"r\", moving camera z forwards by {}.",
                        MOVE_CAMERA_BY
                    );

                    inner_locked.camera_position.z += MOVE_CAMERA_BY;
                    // inner_locked.camera_look_target.z += MOVE_CAMERA_BY;

                    inner_locked.pending_rerender = true;
                } else if input_key == "t" {
                    info!("User pressed \"t\", zooming outwards.");

                    let camera_position_movement =
                        (inner_locked.camera_look_target - inner_locked.camera_position).normalize()
                            * MOVE_CAMERA_BY;

                    inner_locked.camera_position -= camera_position_movement;

                    inner_locked.pending_rerender = true;
                } else if input_key == "g" {
                    info!("User pressed \"g\", zooming inwards.");

                    let camera_position_movement =
                        (inner_locked.camera_look_target - inner_locked.camera_position).normalize()
                            * MOVE_CAMERA_BY;

                    inner_locked.camera_position += camera_position_movement;

                    inner_locked.pending_rerender = true;
                }

                drop(inner_locked);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if matches!(button, MouseButton::Left) {
                    match state {
                        ElementState::Pressed => {
                            info!("Left mouse button pressed.");

                            self.user_control.left_mouse_pressed = true;
                        }
                        ElementState::Released => {
                            info!("Left mouse button released.");

                            self.user_control.left_mouse_pressed = false;
                        }
                    }
                }
            }
            WindowEvent::CursorLeft { .. } => {
                info!("Cursor has left the window.");

                self.user_control.left_mouse_pressed = false;
            }
            _ => {}
        };

        Ok(())
    }
}
