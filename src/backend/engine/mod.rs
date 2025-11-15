//! This module is responsible for the heart of the fluid simulation - the velocity field
//! computation.

use std::{cell::OnceCell, sync::{Arc, mpsc::Sender}, time::Duration};

use crate::backend::pool::{Fish, Pool};

/// The final output of the [`SimulationEngine`] struct when it computes a single
/// simulation timestep.
/// 
/// It is expected to be an expensive struct to allocate, so only do so when necessary. 
#[derive(Debug)]
pub struct EngineOutput; // TODO: Declare fields as necessary

impl EngineOutput {
    /// Initializes a new `EngineOutput` object.
    /// 
    /// This operation is considered expensive, and should only be done if the caller
    /// has no access to an already allocated `EngineOutput`.
    /// 
    /// # Return Value:
    /// A new `EngineOutput` object.
    pub fn new() -> Self {
        // TODO - As the physics computation is implemented, add fields and arguments
        Self {}
    }
}

/// The engine that computes the fluid simulation's
pub struct SimulationEngine {
    engine_output_pool: Arc<Pool<EngineOutput>>,
}

impl SimulationEngine {
    /// Creates a new simulation engine that employs efficient memory reuse by using the given pool.
    /// 
    /// # Arguments:
    /// * `pool` - A pool of [`EngineOutput`] objects, wrapped in an `Arc` so that other threads
    ///            can populate it as well.
    ///            The pool doesn't have to contain any values when passed to the function. The
    ///            engine will allocate a new object if the pool is empty.
    /// 
    /// # Return Value:
    /// A new `SimulationEngine` object that attempts to fish from the given pool.
    pub fn new(pool: Arc<Pool<EngineOutput>>) -> Self {
        Self { engine_output_pool: pool, }
    }

    /// Retrieves a new [`EngineOutput`] object.
    /// 
    /// The object will be retrieved from the pool. If the `wait_for_pool` flag is `true`, the 
    /// function will block until an `EngineOutput` object is available and return it.
    /// 
    /// If `false` however, the function will attempt to retrieve it immediately, and if one isn't
    /// available a new one will be allocated.
    /// 
    /// # Arguments:
    /// * `wait_for_pool` - A flag indicating whether the function should block until the pool can
    ///                     provide an `EngineOutput` object, or a new one should be allocated in
    ///                     the case it can't.
    /// 
    /// # Return Value:
    /// A new `EngineOutput` object wrapped in a fish. If `wait_for_pool` is `true`, it is guaranteed that no
    /// memory allocations will be performed (at least in this thread).
    fn get_engine_output(&self, wait_for_pool: bool) -> Fish<EngineOutput> {
        if wait_for_pool {
            self.engine_output_pool.get_fish_blocking()
        } else {
            self.engine_output_pool.get_fish_or_init(EngineOutput::new)
        }
    }

    /// Computes the next timestep in the simulation, given the last one and the amount of time that passed
    /// between them.
    /// 
    /// # Arguments:
    /// * `dt` - The difference in time between the previous timestep and the timestep that the caller wishes
    ///          to compute.
    /// * `prev_timestep` - The previous state of the simulation, will directly affect the returned timestep.
    /// * `wait_for_pool` - A flag indicating whether the function should block until the engine's pool can
    ///                     provide an `EngineOutput` object to write to, or a new one should be allocated in
    ///                     the case the pool is empty.
    /// 
    /// # Return Value:
    /// A new timestep in the simulation based on the previous one and the amount of time that passed between
    /// them.
    pub fn compute_timestep(&self, dt: Duration, prev_timestep: &EngineOutput, wait_for_pool: bool) -> Fish<EngineOutput> {
        let mut output = self.get_engine_output(wait_for_pool);
        self.compute_timestep_internal(dt, prev_timestep, &mut output);
        output
    }

    /// The actual physics of the engine.
    fn compute_timestep_internal(&self, dt: Duration, prev_timestep: &EngineOutput, output_buffer: &mut EngineOutput) {
        todo!("Implement actual physics here!")
    }
}