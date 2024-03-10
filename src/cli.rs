//! Command-line interface definitions for the server binary.

use std::path::PathBuf;

use clap::Parser;


/// Command-line arguments.
#[derive(Parser)]
#[command(
    name = "nrg-dn1",
    author,
    about = "First homework for NRG (napredna računalniška grafika).",
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

    #[arg(short = 'i', long = "input-file-path")]
    pub input_file_path: PathBuf,
}
