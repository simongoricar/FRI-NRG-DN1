pub trait PixelSurfaceRenderer {
    fn draw(&self, frame: &mut [u8]);
}


pub struct SplatRenderer {
    render_width: u32,
    render_height: u32,
}

impl SplatRenderer {
    pub fn new(render_width: u32, render_height: u32) -> Self {
        Self {
            render_width,
            render_height,
        }
    }
}

impl PixelSurfaceRenderer for SplatRenderer {
    fn draw(&self, frame: &mut [u8]) {
        // TODO
        for (pixel_index, pixel) in frame.chunks_exact_mut(4).enumerate() {
            pixel.copy_from_slice(&[133, 255, 211, 255]);
        }
    }
}
