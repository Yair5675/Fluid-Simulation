//! Handles all logic of the fluid simulation including physics, simulation configuration, etc...

use std::sync::{Arc, RwLock, atomic::AtomicBool};

use self::configuration::BackendConfiguration;
use crate::ipc::SimulationData;

pub mod configuration;
pub mod engine;
pub mod generic_sender;
pub mod grid;
pub mod pool;
pub mod processor;

/// A struct encapsulating the entire simulation's backend. Offers an API to the frontend
/// while simultaneously handling all components of the backend computation chain.
pub struct FluidSimulationBackend {
    is_running: AtomicBool,
    config: BackendConfiguration,
    currently_rendering_data: RwLock<Arc<SimulationData>>,
}

impl FluidSimulationBackend {
    /// Initializes the simulation's backend given its configuration.
    ///
    /// # Arguments:
    /// * `configuration` - The backend's configuration, allows the caller to change the
    ///                     backend's behavior.
    ///
    /// # Return Value:
    /// A new `FluidSimulationBackend`, configured as requested.
    pub fn new(configuration: BackendConfiguration) -> Self {
        Self {
            is_running: AtomicBool::new(false),
            config: configuration,
            currently_rendering_data: RwLock::new(Arc::new(SimulationData::Loading)),
        }
    }
}
