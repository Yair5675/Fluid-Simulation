//! Module containing implementation of different adapters that can be used in the simulation.

mod raw;

use crate::backend::engine::EngineOutput;
use crate::backend::pool::Fish;
pub use crate::backend::processor::adapters::raw::RawOutputAdapter;
use crate::backend::processor::SimulationDataAdapter;
use crate::ipc::SimulationData;
use anyhow::anyhow;
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
pub enum AdapterFactory {
    RawAdapter(RawOutputAdapter),
}

impl SimulationDataAdapter for AdapterFactory {
    type AdapterError = anyhow::Error;

    fn to_simulation_data(
        self,
        prev_state: &SimulationData,
        engine_output: Arc<Fish<EngineOutput>>,
    ) -> Result<SimulationData, Self::AdapterError> {
        match self {
            AdapterFactory::RawAdapter(adapter) => adapter
                .to_simulation_data(prev_state, engine_output)
                .map_err(|_| anyhow!("Impossible - RawAdapter failed")),
        }
    }

    fn from_simulation_data(
        simulation_data: SimulationData,
    ) -> Result<Self, Self::AdapterError> {
        match simulation_data {
            SimulationData::EngineOutput(_) => {
                Ok(AdapterFactory::RawAdapter(RawOutputAdapter))
            }
            _ => Err(anyhow!("AdapterFactory doesn't support the given simulation data"))
        }
    }
}
