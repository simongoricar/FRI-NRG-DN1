use std::path::Path;

use bytes::{Buf, Bytes};
use miette::{miette, Context, IntoDiagnostic, Result};
use nalgebra::{Vector3, Vector4};
use rayon::iter::{ParallelBridge, ParallelIterator};


#[derive(Clone, PartialEq, Debug)]
pub struct SplatEntry {
    position: Vector3<f32>,
    scale: Vector3<f32>,
    color: Vector4<u8>,
    rotation: Vector4<f32>,
}

impl SplatEntry {
    fn from_raw_splat_data(mut bytes: Bytes) -> Result<Self> {
        // Structure is 32 bytes big:
        // - position (3x f32)
        // - scale (3x f32)
        // - color (RGBA; 4x u8)
        // - rotation (quarterion components (c-128)/128 ; 4x u8)

        if bytes.len() != 32 {
            return Err(miette!(
                "Provided Bytes container is NOT 32 BYTES BIG!"
            ));
        }


        let position = {
            let position_x = bytes.get_f32();
            let position_y = bytes.get_f32();
            let position_z = bytes.get_f32();

            Vector3::new(position_x, position_y, position_z)
        };

        let scale = {
            let scale_x = bytes.get_f32();
            let scale_y = bytes.get_f32();
            let scale_z = bytes.get_f32();

            Vector3::new(scale_x, scale_y, scale_z)
        };

        let color = {
            let red = bytes.get_u8();
            let green = bytes.get_u8();
            let blue = bytes.get_u8();
            let straight_alpha = bytes.get_u8();

            Vector4::new(red, green, blue, straight_alpha)
        };

        let rotation = {
            let first_raw = bytes.get_u8();
            let second_raw = bytes.get_u8();
            let third_raw = bytes.get_u8();
            let fourth_raw = bytes.get_u8();

            let first_decoded = (first_raw - 128) as f32 / 128f32;
            let second_decoded = (second_raw - 128) as f32 / 128f32;
            let third_decoded = (third_raw - 128) as f32 / 128f32;
            let fourth_decoded = (fourth_raw - 128) as f32 / 128f32;

            Vector4::new(
                first_decoded,
                second_decoded,
                third_decoded,
                fourth_decoded,
            )
        };


        Ok(Self {
            position,
            scale,
            color,
            rotation,
        })
    }
}


#[derive(Clone, PartialEq, Debug)]
pub struct SplatFile {
    pub splats: Vec<SplatEntry>,
}

impl SplatFile {
    pub fn load_from_file<P>(input_file_path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let file_contents = {
            let bytes_vec = std::fs::read(input_file_path.as_ref())
                .into_diagnostic()
                .wrap_err("Failed to read input file.")?;

            Bytes::from(bytes_vec)
        };

        if file_contents.len() % 32 != 0 {
            return Err(miette!(
                "Invalid file: not divisible by 32 bytes!"
            ));
        }

        let parsed_splats = file_contents
            .chunks(32)
            .par_bridge()
            .map(|chunk| SplatEntry::from_raw_splat_data(Bytes::copy_from_slice(chunk)))
            .collect::<Result<Vec<_>>>()?;


        Ok(Self {
            splats: parsed_splats,
        })
    }
}
