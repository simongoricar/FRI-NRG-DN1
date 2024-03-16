use clap::Parser;
use miette::{miette, Context, Result};
use nalgebra::{Vector3, Vector4};
use tracing::info;

use crate::{
    cli::{parse_str_as_point3, parse_str_as_vector3, CLIArgs},
    configuration::Configuration,
    logging::initialize_tracing,
    renderer::SplatRenderer,
    splat_decoder::{Splat, Splats},
};

mod cli;
mod configuration;
mod logging;
mod renderer;
mod splat_decoder;

#[cfg(feature = "ui")]
mod drawing;


/// Splats are parsed from raw data in parallel, resulting in a vector of splats that is non-deterministic.
/// If you wish to manually reorder the splats back to their file order, specify this to be true.
///
/// If this is `false`, the program will not process the file deterministically.
///
/// **This is enabled on debug builds and disabled in release builds.**
#[cfg(debug_assertions)]
pub const REORDER_SPLATS_TO_FILE_ORDER: bool = true;

#[cfg(not(debug_assertions))]
pub const REORDER_SPLATS_TO_FILE_ORDER: bool = false;


/***
 * Compile-time configuration values
 */

pub const DEFAULT_WINDOW_WIDTH: u32 = 720;
pub const DEFAULT_WINDOW_HEIGHT: u32 = 720;


/***
 * END OF compile-time configuration values
 */



/// Construct and return [`Splats`] containing a simple 5-point splatting testing scene.
pub fn get_testing_splat_scene() -> Splats {
    Splats::from_entries(vec![
        Splat::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            Vector4::new(244, 130, 80, 220),
            Vector4::new(0.0, 0.0, 0.0, 0.0),
        ),
        Splat::new(
            Vector3::new(0.1, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            Vector4::new(200, 22, 1, 123),
            Vector4::new(0.0, 0.0, 0.0, 0.0),
        ),
        Splat::new(
            Vector3::new(0.0, 0.1, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            Vector4::new(200, 255, 255, 22),
            Vector4::new(0.0, 0.0, 0.0, 0.0),
        ),
        Splat::new(
            Vector3::new(0.0, 0.0, 0.1),
            Vector3::new(1.0, 1.0, 1.0),
            Vector4::new(22, 255, 255, 90),
            Vector4::new(0.0, 0.0, 0.0, 0.0),
        ),
        Splat::new(
            Vector3::new(0.0, -0.1, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            Vector4::new(22, 2, 255, 100),
            Vector4::new(0.0, 0.0, 0.0, 0.0),
        ),
    ])
}




fn main() -> Result<()> {
    // Parse command-line arguments.
    let cli_args = CLIArgs::parse();


    // Parse configuration file.
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
        "Configuration loaded from \"{}\".",
        configuration.file_path.display()
    );

    configuration
        .screenshot
        .create_screenshot_directory_if_not_exists()?;


    let logging_raii_guard = initialize_tracing(
        configuration.logging.console_output_level_filter(),
        configuration.logging.log_file_output_level_filter(),
        &configuration.logging.log_file_output_directory,
        "nrg-dn1.log",
    )
    .wrap_err("Failed to initialize tracing.")?;

    info!("Tracing initialized.");



    // Load splat data from file if provided, otherwise use the testing scene.
    let splat_data = match cli_args.input_file_path.as_ref() {
        Some(splat_file_path) => Splats::load_from_file(splat_file_path).wrap_err_with(|| {
            miette!(
                "Failed to load splat input file: {}",
                splat_file_path.display()
            )
        })?,
        None => get_testing_splat_scene(),
    };


    // Parse initial rendering parameters from the command-line parameters.
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


    let render_width = cli_args.render_width.unwrap_or(DEFAULT_WINDOW_WIDTH);
    let render_height = cli_args.render_height.unwrap_or(DEFAULT_WINDOW_HEIGHT);


    // Initialize the splat rendered and drawing manager.
    let splat_renderer = SplatRenderer::new(
        configuration,
        render_width,
        render_height,
        splat_data,
        cli_args.splat_scaling_factor,
        initial_camera_position,
        initial_camera_look_target,
        initial_up_vector,
    );

    splat_renderer.render_in_place();


    #[cfg(feature = "ui")]
    {
        if cli_args.export_screenshot_and_exit {
            splat_renderer.save_screenshot_to_disk();
        } else {
            use crate::drawing::WindowManager;

            let drawing_manager = WindowManager::new(render_width, render_height, splat_renderer)
                .wrap_err("Failed to initialize window manager.")?;

            drawing_manager.run()?;
        }
    }

    #[cfg(not(feature = "ui"))]
    {
        // Since all graphical window dependencies are not present,
        // just save a screenshot to disk and exit.
        splat_renderer.save_screenshot_to_disk();
    }



    drop(logging_raii_guard);
    Ok(())
}
