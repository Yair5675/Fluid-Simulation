//! Responsible for containing data types, structs and traits which define and abstract the
//! communication between the frontend module and the backend module.
//!
//! This isn't technically an IPC since both run on the same process, but represents the same idea.

use crate::backend::grid::Grid;

/// Represents the different types of simulation data which can be sent from the backend to the
/// frontend.
pub enum SimulationData {
    /// No data is available.
    Loading,
    /// A grid of pressure values.
    Pressure(Grid<f64>),
}

impl Default for SimulationData {
    fn default() -> Self {
        Self::Loading
    }
}
