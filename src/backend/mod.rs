//! Handles all logic of the fluid simulation including physics, simulation configuration, etc...

pub mod configuration;
pub mod engine;
pub mod grid;
pub mod pool;

use crate::ipc::SimulationData;
use anyhow::Result;
use std::time::Duration;

/// A trait that exposes the API of the fluid simulation while abstracting the complex logic
/// and physics.
pub trait FluidSimulationBackend {
    /// Updates the simulation's data according to the backend's logic.
    ///
    /// # Arguments:
    /// * `delta_time` - The amount of time that had passed since the last call to
    ///                  `update_simulation`. Used in the physics equations of the simulation.
    ///
    /// # Return Value:
    /// A simple `anyhow::Result` object, indicating whether the update was successful.
    fn update_simulation(&mut self, delta_time: Duration) -> Result<()>;

    /// Retrieves the computed data from the simulation.
    ///
    /// # Return Value:
    /// A `SimulationData` object, containing the most recent computed values in the backend.
    fn get_simulation_data(&self) -> SimulationData;
}
