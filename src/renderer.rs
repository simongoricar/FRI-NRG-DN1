use std::sync::RwLock;

use miette::Result;
use nalgebra::{Matrix4, Perspective3, Point3, Vector3, Vector4};
use tracing::{debug, info};
use winit::{
    event::{ElementState, WindowEvent},
    keyboard::Key,
};

use crate::splat_decoder::SplatFile;

pub trait PixelSurfaceRenderer {
    fn draw(&self, frame: &mut [u8]);
}

pub trait InteractiveRenderer {
    fn handle_window_event(&mut self, window_evnt: &WindowEvent) -> Result<()>;
}



#[inline]
fn pixel_coordinates_from_projected_coordinates(
    projected_position: Vector4<f32>,
    render_width: u32,
    render_height: u32,
) -> Option<(u32, u32)> {
    let mut projected_x = *projected_position.get(0).unwrap();
    let mut projected_y = *projected_position.get(1).unwrap();
    let mut projected_z = *projected_position.get(2).unwrap();
    let projected_w = *projected_position.get(3).unwrap();

    projected_x /= 1f32 / projected_w;
    projected_y /= 1f32 / projected_w;
    projected_z /= 1f32 / projected_w;

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
    let render_x: u32 = ((projected_x - -1.0) * (render_width - 1) as f32 / 2.0).round() as u32;
    let render_y: u32 = ((projected_y - -1.0) * (render_height - 1) as f32 / 2.0).round() as u32;


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


pub struct SplatRenderer {
    render_width: u32,

    render_height: u32,

    splat_file: SplatFile,

    camera_position: Point3<f32>,

    data: RwLock<Vec<u8>>,
}

impl SplatRenderer {
    pub fn new(render_width: u32, render_height: u32, splat_file: SplatFile) -> Self {
        let data = vec![0; render_width as usize * render_height as usize * 4];
        let data_rwlocked = RwLock::new(data);

        Self {
            render_width,
            render_height,
            splat_file,
            camera_position: Point3::new(0.0, 0.0, 0.0),
            data: data_rwlocked,
        }
    }

    pub fn render_in_place(&self) {
        info!("Rendering splat.");

        // Transform the world coordinates of each splat to camera coordinates.
        let mut splats = self.splat_file.splats.clone();

        let average_splat_position: Point3<f32> = {
            let mut total_position = Point3::new(0f32, 0f32, 0f32);

            for splat in &splats {
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

        debug!(
            "Average splat position: {:?}",
            average_splat_position
        );


        let look_at_matrix = Matrix4::<f32>::look_at_lh(
            &self.camera_position,
            &average_splat_position,
            &Vector3::new(0f32, 1f32, 0f32),
        );

        let projection_matrix = Perspective3::<f32>::new(
            self.render_width as f32 / self.render_height as f32,
            80f32,
            0.1,
            100.0,
        );


        let mut data_locked = self
            .data
            .write()
            .expect("a thread panicked while holding a RwLock of renderer data");

        // Reset canvas.
        for some_pixel_component in data_locked.iter_mut() {
            *some_pixel_component = 0;
        }

        // Draw to canvas.
        for splat in splats.iter_mut() {
            let position_in_world_space = Vector4::new(
                splat.position.x,
                splat.position.y,
                splat.position.z,
                1f32,
            );

            let position_in_camera_space = look_at_matrix * position_in_world_space;
            let position_in_clipspace = projection_matrix.as_matrix() * position_in_camera_space;


            if let Some((render_x, render_y)) = pixel_coordinates_from_projected_coordinates(
                position_in_clipspace,
                self.render_width,
                self.render_height,
            ) {
                let pixel_index = ((render_y * self.render_width + render_x) * 4) as usize;

                data_locked.as_mut_slice()[pixel_index..pixel_index + 4]
                    .copy_from_slice(splat.color.as_slice());
            }
        }
    }
}

impl PixelSurfaceRenderer for SplatRenderer {
    fn draw(&self, frame: &mut [u8]) {
        // self.render_in_place();

        {
            let data_read_only = self
                .data
                .read()
                .expect("a thread panicked while holding a RwLock of renderer data");

            frame.copy_from_slice(&data_read_only);
        }

        // for (pixel_index, pixel) in frame.chunks_exact_mut(4).enumerate() {
        //      pixel.copy_from_slice(&[133, 255, 211, 255]);
        // }
    }
}


impl InteractiveRenderer for SplatRenderer {
    fn handle_window_event(&mut self, window_event: &WindowEvent) -> Result<()> {
        let WindowEvent::KeyboardInput { event, .. } = window_event else {
            return Ok(());
        };

        if event.state != ElementState::Released {
            return Ok(());
        }

        let Key::Character(input_key) = &event.logical_key else {
            return Ok(());
        };


        let mut needs_rerender = false;
        const MOVE_CAMERA_BY: f32 = 0.2;

        if input_key == "s" {
            info!(
                "User pressed \"s\", moving camera x backwards by {}.",
                MOVE_CAMERA_BY
            );

            self.camera_position.x -= MOVE_CAMERA_BY;
            needs_rerender = true;
        } else if input_key == "w" {
            info!(
                "User pressed \"w\", moving camera x forwards by {}.",
                MOVE_CAMERA_BY
            );

            self.camera_position.x += MOVE_CAMERA_BY;
            needs_rerender = true;
        } else if input_key == "d" {
            info!(
                "User pressed \"d\", moving camera y backwards by {}.",
                MOVE_CAMERA_BY
            );

            self.camera_position.y -= MOVE_CAMERA_BY;
            needs_rerender = true;
        } else if input_key == "e" {
            info!(
                "User pressed \"e\", moving camera y forwards by {}.",
                MOVE_CAMERA_BY
            );

            self.camera_position.y += MOVE_CAMERA_BY;
            needs_rerender = true;
        } else if input_key == "f" {
            info!(
                "User pressed \"f\", moving camera z backwards by {}.",
                MOVE_CAMERA_BY
            );

            self.camera_position.z -= MOVE_CAMERA_BY;
            needs_rerender = true;
        } else if input_key == "r" {
            info!(
                "User pressed \"r\", moving camera z forwards by {}.",
                MOVE_CAMERA_BY
            );

            self.camera_position.z += MOVE_CAMERA_BY;
            needs_rerender = true;
        }

        if needs_rerender {
            self.render_in_place();
        }

        Ok(())
    }
}
