use clap::Parser;
use miette::{miette, Context, Result};
use tracing::info;

use crate::{
    cli::CLIArgs,
    configuration::Configuration,
    logging::initialize_tracing,
    splat_decoder::SplatFile,
};

mod cli;
mod configuration;
mod logging;
mod splat_decoder;

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


    drop(logging_raii_guard);
    Ok(())
}
