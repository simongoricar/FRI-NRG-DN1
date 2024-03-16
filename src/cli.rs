//! Command-line interface definitions for the server binary.

use std::path::PathBuf;

use clap::Parser;
use miette::{miette, Context, IntoDiagnostic, Result};
use nalgebra::{Point3, Vector3};



/// Command-line arguments.
#[derive(Parser)]
#[command(
    name = "nrg-dn1",
    author,
    about = "First homework for NRG - gaussian splatting.",
    version
)]
pub struct CLIArgs {
    /// This is the path to the configuration file to use.
    /// If unspecified, this defaults to `./data/configuration.toml`.
    #[arg(
        short = 'c',
        long = "configuration-file-path",
        help = "Path to the configuration file to use. Defaults to ./data/configuration.toml"
    )]
    pub configuration_file_path: Option<PathBuf>,

    #[arg(
        short = 'i',
        long = "input-file-path",
        help = "*.splat file to use. If unspecified, a small testing scene is shown."
    )]
    pub input_file_path: Option<PathBuf>,

    #[arg(
        long = "export-screenshot-and-exit",
        help = "If this flag is present, the program will perform a single render \
                using the provided parameters, save the screenshot to disk, and exit."
    )]
    pub export_screenshot_and_exit: bool,

    #[arg(
        short = 's',
        long = "splat-scaling-factor",
        help = "Splat perspective closeness scaling factor (float), defaults to 2.0."
    )]
    pub splat_scaling_factor: Option<f32>,

    #[arg(
        long = "camera-position",
        help = "Initial camera position (in world space). Format: \"x,y,z\". \
                If unspecified, this will default to (3,3,3)."
    )]
    pub camera_position: Option<String>,

    #[arg(
        long = "camera-look-target",
        help = "Initial camera look target position (in world space). Format: \"x,y,z\". \
                If unspecified, this will default to the average splat position."
    )]
    pub camera_look_target: Option<String>,

    #[arg(
        long = "initial-up-vector",
        help = "Initial up vector for the camera perspective projection. Format: \"x,y,z\". \
                If unspecified, this will default to (0,1,0)."
    )]
    pub initial_up_vector: Option<String>,

    #[arg(
        long = "render-width",
        help = "Width of the render window / canvas. Defaults to 720 pixels if unspecified."
    )]
    pub render_width: Option<u32>,

    #[arg(
        long = "render-height",
        help = "Height of the render window / canvas. Defaults to 720 pixels if unspecified."
    )]
    pub render_height: Option<u32>,
}


/// Parse a string of the format `1,2.5,3` or `(1,2.0,-3.1)` into
/// a tuple with three `f32` elements (representing `x`, `y`, and `z`).
pub fn parse_str_as_three_f32_points(value: &str) -> Result<(f32, f32, f32)> {
    let value = value.replace(['(', ')'], "");
    let components = value.splitn(3, ',').collect::<Vec<_>>();

    if components.len() != 3 {
        return Err(miette!(
            "Failed to decode string to Point3<f32>: expected format (x,y,z), got {}.",
            value
        ));
    }


    let x_value = components[0]
        .parse::<f32>()
        .into_diagnostic()
        .wrap_err_with(|| {
            miette!(
                "Failed to decode string to f32: expected x coordinate to be valid f32, found {}.",
                components[0]
            )
        })?;

    let y_value = components[1]
        .parse::<f32>()
        .into_diagnostic()
        .wrap_err_with(|| {
            miette!(
                "Failed to decode string to f32: expected y coordinate to be valid f32, found {}.",
                components[1]
            )
        })?;

    let z_value = components[2]
        .parse::<f32>()
        .into_diagnostic()
        .wrap_err_with(|| {
            miette!(
                "Failed to decode string to f32: expected z coordinate to be valid f32, found {}.",
                components[2]
            )
        })?;


    Ok((x_value, y_value, z_value))
}


/// Parse a string of the format `1,2.5,3` or `(1,2.0,-3.1)` into
/// a [`Point3::<f32>`][Point3].
#[inline]
pub fn parse_str_as_point3(value: &str) -> Result<Point3<f32>> {
    let (x, y, z) = parse_str_as_three_f32_points(value)?;
    Ok(Point3::new(x, y, z))
}

/// Parse a string of the format `1,2.5,3` or `(1,2.0,-3.1)` into
/// a [`Vector3::<f32>`][Vector3].
#[inline]
pub fn parse_str_as_vector3(value: &str) -> Result<Vector3<f32>> {
    let (x, y, z) = parse_str_as_three_f32_points(value)?;
    Ok(Vector3::new(x, y, z))
}
