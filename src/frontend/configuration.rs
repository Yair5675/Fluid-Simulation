//! Front end configuration module, offers an API to change how the fluid simulation will be
//! presented.

/// Configuration struct for the frontend.
pub struct FrontendConfiguration {}

impl std::default::Default for FrontendConfiguration {
    /// Provide the default frontend configuration for the fluid simulation.
    fn default() -> Self {
        FrontendConfiguration {}
    }
}