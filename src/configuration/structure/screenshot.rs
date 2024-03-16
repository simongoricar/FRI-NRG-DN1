use std::path::PathBuf;

use miette::{miette, Context, IntoDiagnostic, Result};
use serde::Deserialize;

use super::BasePathsConfiguration;
use crate::configuration::{
    traits::ResolvableConfigurationWithContext,
    utilities::replace_placeholders_in_path,
};


#[derive(Deserialize, Clone, Debug)]
pub(super) struct UnresolvedScreenshotConfiguration {
    screenshot_directory_path: String,
}

#[derive(Clone, Debug)]
pub struct ScreenshotConfiguration {
    pub screenshot_directory_path: PathBuf,
}


impl ResolvableConfigurationWithContext for UnresolvedScreenshotConfiguration {
    type Resolved = ScreenshotConfiguration;
    type Context = BasePathsConfiguration;

    fn resolve(self, context: Self::Context) -> miette::Result<Self::Resolved> {
        let screenshot_directory_path = replace_placeholders_in_path(
            self.screenshot_directory_path,
            context.placeholders_map(),
        );


        Ok(Self::Resolved {
            screenshot_directory_path,
        })
    }
}

impl ScreenshotConfiguration {
    /// Creates the screenshot directory if it does not already exist.
    pub fn create_screenshot_directory_if_not_exists(&self) -> Result<()> {
        std::fs::create_dir_all(&self.screenshot_directory_path)
            .into_diagnostic()
            .wrap_err_with(|| {
                miette!(
                    "Failed to create missing screenshot directory at {}.",
                    self.screenshot_directory_path.display()
                )
            })
    }

    /// Returns a full screenshot path by joining the screenshot directory
    /// and `screenshot_file_name`.
    pub fn screenshot_path(&self, screenshot_file_name: &str) -> PathBuf {
        self.screenshot_directory_path.join(screenshot_file_name)
    }
}
