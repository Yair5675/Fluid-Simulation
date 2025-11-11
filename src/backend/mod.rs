//! Handles all logic of the fluid simulation including physics, simulation configuration, etc...

mod grids;
pub mod data_types;

use std::time::Duration;
use anyhow::Result;
use crate::ipc::SimulationData;

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
