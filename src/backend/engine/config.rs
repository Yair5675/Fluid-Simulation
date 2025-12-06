//! Configuration for the backend's engine.

pub const DEFAULT_PROJECTIONS_ITERATIONS: usize = 25;
pub const DEFAULT_OVERRELAXATION_FACTOR: f64 = 1.9;
pub const DEFAULT_VELOCITY_ABSORPTION_FACTOR: f64 = 0.3;
pub const DEFAULT_STIFFNESS_FACTOR: f64 = 1.0;
pub const DEFAULT_PIC_FACTOR: f64 = 0.1;
pub const DEFAULT_FLIP_FACTOR: f64 = 1.0 - DEFAULT_PIC_FACTOR;

// Wrap in functions due to serde's incompetence...
const fn default_projections_iterations() -> usize {
    DEFAULT_PROJECTIONS_ITERATIONS
}

const fn default_overrelaxation_factor() -> f64 {
    DEFAULT_OVERRELAXATION_FACTOR
}

const fn default_velocity_absorption_factor() -> f64 {
    DEFAULT_VELOCITY_ABSORPTION_FACTOR
}

const fn default_stiffness_factor() -> f64 {
    DEFAULT_STIFFNESS_FACTOR
}

const fn default_pic_factor() -> f64 {
    DEFAULT_PIC_FACTOR
}

const fn default_flip_factor() -> f64 {
    DEFAULT_FLIP_FACTOR
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub struct EngineConfiguration {
    #[serde(default = "default_projections_iterations")]
    pub projection_iterations: usize,
    #[serde(default = "default_overrelaxation_factor")]
    pub overrelaxation_factor: f64,
    #[serde(default = "default_velocity_absorption_factor")]
    pub velocity_absorption_factor: f64,
    #[serde(default = "default_stiffness_factor")]
    pub stiffness_factor: f64,
    #[serde(default = "default_pic_factor")]
    pub pic_factor: f64,
    #[serde(default = "default_flip_factor")]
    pub flip_factor: f64,
}

impl Default for EngineConfiguration {
    fn default() -> Self {
        Self {
            projection_iterations: DEFAULT_PROJECTIONS_ITERATIONS,
            overrelaxation_factor: DEFAULT_OVERRELAXATION_FACTOR,
            velocity_absorption_factor: DEFAULT_VELOCITY_ABSORPTION_FACTOR,
            stiffness_factor: DEFAULT_STIFFNESS_FACTOR,
            pic_factor: DEFAULT_PIC_FACTOR,
            flip_factor: DEFAULT_FLIP_FACTOR,
        }
    }
}
