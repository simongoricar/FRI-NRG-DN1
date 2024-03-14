use clap::Parser;
use miette::{miette, Context, IntoDiagnostic, Result};
use nalgebra::{Point3, Vector3, Vector4};
use tracing::info;

use crate::{
    cli::CLIArgs,
    configuration::Configuration,
    drawing::WindowDrawingManager,
    logging::initialize_tracing,
    renderer::SplatRenderer,
    splat_decoder::{Splat, Splats},
};

mod cli;
mod configuration;
mod drawing;
mod logging;
mod renderer;
mod splat_decoder;


/***
 * Compile-time configuration values
 */

/// Splats are parsed from raw data in parallel, resulting in a vector of splats that is non-deterministic.
/// If you wish to manually reorder the splats back to their file order, specify this to be true.
///
/// If this is `false`, the program will not process the file deterministically.
pub const REORDER_SPLATS_TO_FILE_ORDER: bool = true;

pub const WINDOW_WIDTH: u32 = 720;
pub const WINDOW_HEIGHT: u32 = 720;


/***
 * END OF compile-time configuration values
 */

fn parse_str_as_three_f32_points(value: &str) -> Result<(f32, f32, f32)> {
    let value = value.replace(['(', ')'], "");
    let components = value.splitn(3, ',').collect::<Vec<_>>();

    if components.len() != 3 {
        return Err(miette!(
            "Failed to decode string to Point3<f32>: expected format (x,y,z), got {}.",
            value
        ));
    }


    let x_value = components[0].parse::<f32>()
        .into_diagnostic()
        .wrap_err_with(|| miette!("Failed to decode string to Point3<f32>: expected x coordinate to be valid f32, found {}.", components[0]))?;

    let y_value = components[1].parse::<f32>()
        .into_diagnostic()
        .wrap_err_with(|| miette!("Failed to decode string to Point3<f32>: expected y coordinate to be valid f32, found {}.", components[1]))?;

    let z_value = components[2].parse::<f32>()
        .into_diagnostic()
        .wrap_err_with(|| miette!("Failed to decode string to Point3<f32>: expected z coordinate to be valid f32, found {}.", components[2]))?;


    Ok((x_value, y_value, z_value))
}


#[inline]
fn parse_str_as_point3(value: &str) -> Result<Point3<f32>> {
    let (x, y, z) = parse_str_as_three_f32_points(value)?;
    Ok(Point3::new(x, y, z))
}

#[inline]
fn parse_str_as_vector3(value: &str) -> Result<Vector3<f32>> {
    let (x, y, z) = parse_str_as_three_f32_points(value)?;
    Ok(Vector3::new(x, y, z))
}


fn main() -> Result<()> {
    let cli_args = CLIArgs::parse();

    // Load configuration.
    let configuration = match cli_args.configuration_file_path.as_ref() {
        Some(path) => {
            println!("Loading configuration: {}", path.display());
            Configuration::load_from_path(path)
        }
        None => {
            println!("Loading configuration at default path.");
            Configuration::load_from_default_path()
        }
    }
    .wrap_err("Failed to load configuration file.")?;

    println!(
        "Configuration loaded: {}.",
        configuration.file_path.display()
    );

    if !configuration.screenshot.screenshot_directory_path.exists() {
        std::fs::create_dir_all(&configuration.screenshot.screenshot_directory_path)
            .into_diagnostic()
            .wrap_err_with(|| {
                miette!(
                    "Failed to create missing screenshot directory at {}.",
                    configuration.screenshot.screenshot_directory_path.display()
                )
            })?;
    }


    let logging_raii_guard = initialize_tracing(
        configuration.logging.console_output_level_filter(),
        configuration.logging.log_file_output_level_filter(),
        &configuration.logging.log_file_output_directory,
        "nrg-dn1.log",
    )
    .wrap_err("Failed to initialize tracing.")?;

    info!("Tracing initialized.");


    // TODO

    // let input_file_splats =
    //     SplatFile::load_from_file(&cli_args.input_file_path).wrap_err_with(|| {
    //         miette!(
    //             "Failed to load input file: {}.",
    //             cli_args.input_file_path.display()
    //         )
    //     })?;
    //
    // println!(
    //     "First splat: {:?}",
    //     input_file_splats.splats.first().unwrap()
    // );

    let input_file_splats = Splats::from_entries(vec![
        Splat::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            Vector4::new(244, 130, 80, 255),
            Vector4::new(0.0, 0.0, 0.0, 0.0),
        ),
        Splat::new(
            Vector3::new(0.1, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            Vector4::new(200, 22, 1, 255),
            Vector4::new(0.0, 0.0, 0.0, 0.0),
        ),
        Splat::new(
            Vector3::new(0.0, 0.1, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            Vector4::new(200, 255, 255, 255),
            Vector4::new(0.0, 0.0, 0.0, 0.0),
        ),
        Splat::new(
            Vector3::new(0.0, 0.0, 0.1),
            Vector3::new(1.0, 1.0, 1.0),
            Vector4::new(22, 255, 255, 255),
            Vector4::new(0.0, 0.0, 0.0, 0.0),
        ),
        Splat::new(
            Vector3::new(0.0, -0.1, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            Vector4::new(22, 2, 255, 255),
            Vector4::new(0.0, 0.0, 0.0, 0.0),
        ),
    ]);


    let initial_camera_position = match cli_args.camera_position.as_ref() {
        Some(position_as_string) => Some(parse_str_as_point3(position_as_string)?),
        None => None,
    };

    let initial_camera_look_target = match cli_args.camera_look_target.as_ref() {
        Some(position_as_string) => Some(parse_str_as_point3(position_as_string)?),
        None => None,
    };

    let initial_up_vector = match cli_args.initial_up_vector.as_ref() {
        Some(vector_as_string) => Some(parse_str_as_vector3(vector_as_string)?),
        None => None,
    };


    let splat_renderer = SplatRenderer::new(
        configuration,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        input_file_splats,
        initial_camera_position,
        initial_camera_look_target,
        initial_up_vector,
    );

    splat_renderer.render_in_place();

    let drawing_manager = WindowDrawingManager::new(splat_renderer)
        .wrap_err("Failed to initialize WindowDrawingManager.")?;

    drawing_manager.run()?;


    drop(logging_raii_guard);
    Ok(())
}
