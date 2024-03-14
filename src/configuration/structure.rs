use std::fs;
use std::path::{Path, PathBuf};

use miette::{Context, IntoDiagnostic, Result};
use serde::Deserialize;

pub use self::base_paths::BasePathsConfiguration;
use self::base_paths::UnresolvedBasePathsConfiguration;
pub use self::logging::LoggingConfiguration;
use self::logging::UnresolvedLoggingConfiguration;
pub use self::screenshot::ScreenshotConfiguration;
use self::screenshot::UnresolvedScreenshotConfiguration;
use super::traits::{ResolvableConfiguration, ResolvableConfigurationWithContext};
use super::utilities::get_default_configuration_file_path;

mod base_paths;
mod logging;
mod screenshot;



#[derive(Deserialize, Debug)]
pub(crate) struct UnresolvedConfiguration {
    /// Base paths.
    base_paths: UnresolvedBasePathsConfiguration,

    /// Logging-related configuration.
    logging: UnresolvedLoggingConfiguration,

    /// Screenshotting configuration.
    screenshot: UnresolvedScreenshotConfiguration,
}


/// The entire configuration.
#[derive(Debug, Clone)]
pub struct Configuration {
    /// This is the file path this `Config` instance was loaded from.
    pub file_path: PathBuf,

    /// Base paths
    pub base_paths: BasePathsConfiguration,

    /// Logging-related configuration.
    pub logging: LoggingConfiguration,

    /// Screenshotting configuration.
    pub screenshot: ScreenshotConfiguration,
}


impl ResolvableConfigurationWithContext for UnresolvedConfiguration {
    type Resolved = Configuration;
    type Context = PathBuf;

    fn resolve(self, context: Self::Context) -> Result<Self::Resolved> {
        let base_paths = self
            .base_paths
            .resolve()
            .wrap_err("Failed to resolve base_paths table.")?;

        let logging = self
            .logging
            .resolve(base_paths.clone())
            .wrap_err("Failed to resolve logging table.")?;

        let screenshot = self
            .screenshot
            .resolve(base_paths.clone())
            .wrap_err("Failed ot resolve screenshot table.")?;


        Ok(Configuration {
            base_paths,
            file_path: context,
            logging,
            screenshot,
        })
    }
}


impl Configuration {
    /// Load the configuration from a specific file path.
    pub fn load_from_path<S: AsRef<Path>>(configuration_file_path: S) -> Result<Self> {
        // Read the configuration file into memory.
        let configuration_string = fs::read_to_string(configuration_file_path.as_ref())
            .expect("Could not read configuration file!");


        // Parse the string into the `UnresolvedConfiguration` structure and then resolve it.
        let unresolved_configuration =
            toml::from_str::<UnresolvedConfiguration>(&configuration_string)
                .into_diagnostic()
                .wrap_err("Could not load configuration file!")?;


        let configuration_file_path = dunce::canonicalize(configuration_file_path)
            .into_diagnostic()
            .wrap_err("Could not canonicalize configuration file path!")?;

        let resolved_configuration = unresolved_configuration
            .resolve(configuration_file_path)
            .wrap_err("Failed to resolve configuration.")?;

        Ok(resolved_configuration)
    }

    /// Load the configuration from the default path (`./data/configuration.toml`).
    pub fn load_from_default_path() -> Result<Configuration> {
        Configuration::load_from_path(
            get_default_configuration_file_path()
                .wrap_err_with(|| "Could not load configuration file at default path.")?,
        )
    }
}
