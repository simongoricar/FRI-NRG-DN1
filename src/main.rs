use clap::Parser;
use miette::{miette, Context, Result};
use tracing::info;

use crate::{
    cli::CLIArgs,
    configuration::Configuration,
    drawing::WindowDrawingManager,
    logging::initialize_tracing,
    renderer::SplatRenderer,
    splat_decoder::SplatFile,
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


    let logging_raii_guard = initialize_tracing(
        configuration.logging.console_output_level_filter(),
        configuration.logging.log_file_output_level_filter(),
        &configuration.logging.log_file_output_directory,
        "nrg-dn1.log",
    )
    .wrap_err("Failed to initialize tracing.")?;

    info!("Tracing initialized.");


    // TODO

    let input_file_splats =
        SplatFile::load_from_file(&cli_args.input_file_path).wrap_err_with(|| {
            miette!(
                "Failed to load input file: {}.",
                cli_args.input_file_path.display()
            )
        })?;

    println!(
        "First splat: {:?}",
        input_file_splats.splats.first().unwrap()
    );

    let splat_renderer = SplatRenderer::new(WINDOW_WIDTH, WINDOW_HEIGHT, input_file_splats);
    splat_renderer.render_in_place();

    let drawing_manager = WindowDrawingManager::new(splat_renderer)
        .wrap_err("Failed to initialize WindowDrawingManager.")?;

    drawing_manager.run()?;


    drop(logging_raii_guard);
    Ok(())
}
