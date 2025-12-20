//! Handles all logic of the fluid simulation including physics, simulation configuration, etc...

use self::{
    configuration::BackendConfiguration,
    engine::{EngineOutput, SimulationEngine},
    generic_sender::GenericSender,
    pool::{Fish, Pool},
    processor::{
        adapters::AdapterFactory,
        SimulationOutputProcessor
    }
};
use crate::ipc::SimulationData;
use std::cell::RefCell;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
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
    latest_data: RwLock<Arc<SimulationData>>,
    // Use RefCell since we have to mutate the engine but need to do so from an immutable reference
    // to self
    engine: RefCell<SimulationEngine>,
    engine_output_pool: Arc<Pool<EngineOutput>>,
    processor: SimulationOutputProcessor<AdapterFactory>,
    processor_pool: Arc<Pool<SimulationData>>,
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
        let engine_pool = Arc::new(if let Some(pool_limit) = configuration.engine_pool_limit {
            Pool::new_bounded(pool_limit.get())
        } else {
            Pool::new_unbounded()
        });
        let processor_pool = Arc::new(
            if let Some(pool_limit) = configuration.processor_pool_limit {
                Pool::new_bounded(pool_limit.get())
            } else {
                Pool::new_unbounded()
            },
        );

        let latest_data = Arc::new(SimulationData::Loading);
        Self {
            is_running: AtomicBool::new(false),
            config: configuration,
            currently_rendering_data: RwLock::new(Arc::clone(&latest_data)),
            latest_data: RwLock::new(latest_data),
            engine: RefCell::new(Self::initialize_engine(
                &configuration,
                Arc::clone(&engine_pool),
            )),
            engine_output_pool: engine_pool,
            processor: SimulationOutputProcessor::new(
                Arc::clone(&processor_pool),
                Box::new(move || AdapterFactory::create(configuration.adapter_configuration)),
            ),
            processor_pool,
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

    /// Sets a flag that signals any thread in the backend loop to stop
    pub fn stop_backend_loop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
    }

    fn run_engine_thread_logic(&self, output_sender: GenericSender<Arc<Fish<EngineOutput>>>) {
        let mut prev_timestep = Arc::new(self.engine_output_pool.get_fish_blocking());

        while self.is_running.load(Ordering::Relaxed) {
            let new_timestep = self.engine.borrow_mut().compute_timestep(
                self.config.time_between_frames,
                &prev_timestep,
                false,
            ); // TODO: Configure wait_for_pool later

            // TODO Log errors later
            if let Ok(new_timestep) = new_timestep {
                prev_timestep = Arc::new(new_timestep);
                let send_result = output_sender.send(Arc::clone(&prev_timestep));

                // Send operation can only fail if the sender disconnected. In this case, any results
                // we will produce won't be sent anywhere, so we should stop wasting CPU on them:
                if let Err(_) = send_result {
                    // TODO Log the error
                    break;
                }
            }
        }
    }

    fn run_processor_thread_logic(&self, output_receiver: Receiver<Arc<Fish<EngineOutput>>>, data_sender: GenericSender<Arc<SimulationData>>) {
        while self.is_running.load(Ordering::Relaxed) {
            let output = match output_receiver.recv() {
                Ok(output) => output,
                // A RecvError is returned if every sender disconnected. In such case, we will
                // never receive anything anymore, so we should exit the loop
                Err(_) => break
            };
            let prev_data = self.get_latest_simulation_data();
            let processed_data = self.processor.process_engine_output(prev_data.as_ref(), output);
            match processed_data {
                Ok(data) => {
                    let data = Arc::new(data);
                    self.set_data_as_latest(Arc::clone(&data));
                    if let Err(_) = data_sender.send(data) {
                        // Like the RecvError, a SendError will be returned only if the sender
                        // disconnected from its receiver, and we can't send anything anymore
                        break;
                    }
                }
                Err(_) => {
                    // TODO Log error
                }
            }
        }
    }

    fn get_latest_simulation_data(&self) -> Arc<SimulationData> {
        match self.latest_data.read() {
            Ok(data) => Arc::clone(&data),
            Err(lock_error) => {
                // TODO Log the lock error later. It indicates a thread panicked while holding a lock
                let data = lock_error.into_inner();
                self.latest_data.clear_poison();
                Arc::clone(&data)
            }
        }
    }

    fn set_data_as_latest(&self, new_latest_data: Arc<SimulationData>) {
        match self.latest_data.write() {
            Ok(mut latest_data_lock) => {
                *latest_data_lock = new_latest_data;
            }
            Err(lock_error) => {
                // TODO log the error
                // We don't care about the previous latest data so we can just clear the poison
                self.latest_data.clear_poison();
                let mut latest_data_lock = lock_error.into_inner();
                *latest_data_lock = new_latest_data;
            }
        }
    }

    fn run_publishing_queue_thread_logic(&self, received_data_queue: Receiver<Arc<SimulationData>>) {
        while self.is_running.load(Ordering::Relaxed) {
            let latest_data = match received_data_queue.recv() {
                Ok(data) => data,
                // A RecvError is returned if every sender disconnected. In such case, we will
                // never receive anything anymore, so we should exit the loop
                Err(_) => break,
            };
            match self.currently_rendering_data.write() {
                Ok(mut current_data) => {
                    *current_data = latest_data;
                }
                Err(lock_error) => {
                    // TODO log the error
                    // We don't care about the previous latest data so we can just clear the poison
                    self.currently_rendering_data.clear_poison();
                    let mut latest_data_lock = lock_error.into_inner();
                    *latest_data_lock = latest_data;
                }
            }
        }
    }
}
