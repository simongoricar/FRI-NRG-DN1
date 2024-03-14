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

    #[arg(
        long = "camera-position",
        help = "Initial camera position (world space). Format: \"x,y,z\". \
                If unspecified, this will default to (3,3,3)."
    )]
    pub camera_position: Option<String>,

    #[arg(
        long = "camera-look-target",
        help = "Initial camera look target position (world space). Format: \"x,y,z\". \
                If unspecified, this will default to the average splat position."
    )]
    pub camera_look_target: Option<String>,

    #[arg(
        long = "initial-up-vector",
        help = "Initial up vector for the camera perspective projection. Format: \"x,y,z\". \
                If unspecified, this will default to (0,1,0)."
    )]
    pub initial_up_vector: Option<String>,
}
