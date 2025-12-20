//! Module containing implementation of different adapters that can be used in the simulation.

use crate::backend::engine::EngineOutput;
use crate::backend::pool::Fish;
use crate::backend::processor::SimulationDataAdapter;
use serde::Deserialize;
use std::sync::Arc;

/// An adapter-specific configuration, through which the [`AdapterFactory`] enum will be able to construct
/// the needed adapter while abstracting which one it provided.
#[derive(Debug, Clone, Copy, Deserialize)]
pub enum AdapterConfiguration {}

/// A factory enum whose variants are wrappers for actual implementations of the [`SimulationDataAdapter`]
/// trait.
///
/// The factory will use the [`AdapterConfiguration`] given to it to create the matching adapter variant.
pub enum AdapterFactory {}

impl SimulationDataAdapter for AdapterFactory {
    type AdapterError = anyhow::Error;

    fn to_simulation_data(
        self,
        prev_state: &crate::ipc::SimulationData,
        engine_output: Arc<Fish<EngineOutput>>,
    ) -> Result<crate::ipc::SimulationData, Self::AdapterError> {
        todo!("Delegate call to actual adapters once they are implemented");
    }

    fn from_simulation_data(
        simulation_data: crate::ipc::SimulationData,
    ) -> Result<Self, Self::AdapterError> {
        todo!("Delegate call to actual adapters once they are implemented");
    }
}
