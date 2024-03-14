use std::path::PathBuf;

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
