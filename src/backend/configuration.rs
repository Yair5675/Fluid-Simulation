//! Defines the configurable attributes of the simulation's backend.

use std::num::NonZeroUsize;

use serde::Deserialize;

use crate::backend::{engine::EngineConfiguration, processor::adapters::AdapterConfiguration};

/// A struct grouping all configurable attributes of the simulation, allowing
/// its behavior to be changed flexibly.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct BackendConfiguration {
    /// Number of particles simulated
    pub particles_count: NonZeroUsize,

    /// Horizontal dimension of the grid used (number of cells in the horizontal dimension)
    pub grid_width: NonZeroUsize,

    /// Vertical dimension of the grid used (number of cells in the vertical dimension)
    pub grid_height: NonZeroUsize,

    /// The physical size of each cell's side in the grid (the distance between one cell and
    /// another)
    pub grid_spacing: f64,

    /// Maximum size of the pool used by the backend's engine.
    ///
    /// If None, the pool can be arbitrarily large.
    pub engine_pool_limit: Option<NonZeroUsize>,

    /// Maximum size of the pool used by the backend's simulation output processor.
    ///
    /// If None, the pool can be arbitrarily large.
    pub processor_pool_limit: Option<NonZeroUsize>,

    /// Configuration for the specific adapter used in the simulation:
    pub adapter_configuration: AdapterConfiguration,

    /// Configurable values specific to the simulation's engine:
    pub engine: EngineConfiguration,
}
