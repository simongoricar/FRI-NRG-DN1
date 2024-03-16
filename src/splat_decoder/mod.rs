use std::{path::Path, time::Instant};

use bytes::{Buf, Bytes};
use miette::{miette, Context, IntoDiagnostic, Result};
use nalgebra::{Vector3, Vector4};
use rayon::iter::{ParallelBridge, ParallelIterator};
use tracing::trace;

use crate::REORDER_SPLATS_TO_FILE_ORDER;


#[derive(Clone, PartialEq, Debug)]
pub struct Splat {
    pub position: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub color: Vector4<u8>,
    pub rotation: Vector4<f32>,
}

impl Splat {
    pub fn new(
        position: Vector3<f32>,
        scale: Vector3<f32>,
        color: Vector4<u8>,
        rotation: Vector4<f32>,
    ) -> Self {
        Self {
            position,
            scale,
            color,
            rotation,
        }
    }

    fn from_raw_splat_file_data(mut bytes: Bytes) -> Result<Self> {
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
            let position_x = bytes.get_f32_le();
            let position_y = bytes.get_f32_le();
            let position_z = bytes.get_f32_le();

            Vector3::new(position_x, position_y, position_z)
        };

        let scale = {
            let scale_x = bytes.get_f32_le();
            let scale_y = bytes.get_f32_le();
            let scale_z = bytes.get_f32_le();

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

            let first_decoded = (first_raw as i32 - 128i32) as f32 / 128f32;
            let second_decoded = (second_raw as i32 - 128i32) as f32 / 128f32;
            let third_decoded = (third_raw as i32 - 128i32) as f32 / 128f32;
            let fourth_decoded = (fourth_raw as i32 - 128i32) as f32 / 128f32;

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
pub struct Splats {
    pub splats: Vec<Splat>,
}

impl Splats {
    pub fn from_entries(splats: Vec<Splat>) -> Self {
        Self { splats }
    }

    #[allow(dead_code)]
    pub fn load_from_file<P>(input_file_path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let time_before_file_read = Instant::now();


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


        trace!(
            "Reading the input file took {} milliseconds.",
            time_before_file_read.elapsed().as_secs_f64() * 1000f64
        );

        let time_before_splats_parse = Instant::now();

        let parsed_splats = if REORDER_SPLATS_TO_FILE_ORDER {
            let mut enumerated_parsed_splats = file_contents
                .chunks(32)
                .enumerate()
                .par_bridge()
                .map(|(chunk_index, chunk)| {
                    let splat = Splat::from_raw_splat_file_data(Bytes::copy_from_slice(chunk))?;

                    Ok((chunk_index, splat))
                })
                .collect::<Result<Vec<_>>>()?;


            trace!(
                "Parsing splats from raw data took {} milliseconds.",
                time_before_splats_parse.elapsed().as_secs_f64() * 1000f64
            );

            let time_before_splats_reordered = Instant::now();


            enumerated_parsed_splats.sort_unstable_by_key(|(chunk_index, _)| *chunk_index);

            let parsed_splats = enumerated_parsed_splats
                .into_iter()
                .map(|(_, splat)| splat)
                .collect::<Vec<_>>();


            trace!(
                "Reordering splats to the original file order took {} milliseconds.",
                time_before_splats_reordered.elapsed().as_secs_f64() * 1000f64
            );

            parsed_splats
        } else {
            let parsed_splats = file_contents
                .chunks(32)
                .par_bridge()
                .map(|chunk| Splat::from_raw_splat_file_data(Bytes::copy_from_slice(chunk)))
                .collect::<Result<Vec<_>>>()?;

            trace!(
                "Parsing splats from raw data took {} milliseconds.",
                time_before_splats_parse.elapsed().as_secs_f64() * 1000f64
            );

            parsed_splats
        };


        Ok(Self {
            splats: parsed_splats,
        })
    }
}
