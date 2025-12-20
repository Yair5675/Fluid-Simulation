use std::sync::Arc;
use crate::backend::engine::EngineOutput;
use crate::backend::pool::Fish;
use crate::backend::processor::SimulationDataAdapter;
use crate::ipc::SimulationData;

/// Simplest kind of [`SimulationDataAdapter`] - it simply returns [`SimulationData::EngineOutput`]
/// wrapping the engine output given to it.
pub struct RawOutputAdapter;

impl SimulationDataAdapter for RawOutputAdapter {
    type AdapterError = ();

    fn to_simulation_data(self, prev_state: &SimulationData, engine_output: Arc<Fish<EngineOutput>>) -> Result<SimulationData, Self::AdapterError> {
        Ok(SimulationData::EngineOutput(engine_output))
    }

    fn from_simulation_data(simulation_data: SimulationData) -> Result<Self, Self::AdapterError>
    where
        Self: Sized
    {
        Ok(RawOutputAdapter)
    }
}