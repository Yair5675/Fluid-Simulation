//! Handles all logic of the fluid simulation including physics, simulation configuration, etc...

use std::cell::RefCell;
use self::{
    configuration::BackendConfiguration,
    engine::{EngineOutput, SimulationEngine},
    pool::Pool,
};
use crate::ipc::SimulationData;
use std::sync::{atomic::AtomicBool, Arc, RwLock};

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
    // Use RefCell since we have to mutate the engine but need to do so from an immutable reference
    // to self
    engine: RefCell<SimulationEngine>,
    engine_output_pool: Arc<Pool<EngineOutput>>,
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
        let pool = Arc::new(if let Some(pool_limit) = configuration.engine_pool_limit {
            Pool::new_bounded(pool_limit.get())
        } else {
            Pool::new_unbounded()
        });

        Self {
            is_running: AtomicBool::new(false),
            config: configuration,
            currently_rendering_data: RwLock::new(Arc::new(SimulationData::Loading)),
            engine: RefCell::new(Self::initialize_engine(&configuration, Arc::clone(&pool))),
            engine_output_pool: pool,
        }
    }

    fn initialize_engine(
        config: &BackendConfiguration,
        engine_pool: Arc<Pool<EngineOutput>>,
    ) -> SimulationEngine {
        SimulationEngine::new(
            config.particles_count.get(),
            config.grid_width.get(),
            config.grid_height.get(),
            config.grid_spacing,
            engine_pool,
            config.engine,
        )
    }
}
