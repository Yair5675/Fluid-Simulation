//! A module responsible to process the `engine` module's output, providing a mechanism
//! that adds information on top of the velocity field computed in the engine.

use std::sync::Arc;

use crate::{
    backend::{
        engine::EngineOutput,
        pool::{Fish, Pool},
    },
    ipc::SimulationData,
};

/// A generic adaptor trait that allows the backend to extract different kinds of information
/// from a single [`EngineOutput`].
pub trait SimulationDataAdapter: TryFrom<SimulationData> + Send {
    type AdapterError;

    /// Computes new simulation data based on the given `engine_output` and returns it.
    ///
    /// The function consumes self to avoid allocating more memory every time the adapter is used.
    /// The adapter's memory is passed to the [`SimulationData`] variant it returns, and can be reused
    /// through the [`TryFrom`] implementation of the adapter, which should succeed only for the
    /// specific variant it produces.
    ///
    /// # Arguments:
    /// * `prev_state` - The previous state of the adapter, represented as a [`SimulationData`] object.
    ///                  The data object must be of the variant mapped to the adapter. If not, the adapter
    ///                  should return an error.
    /// * `engine_output` - The output produced by the `engine` module. The adapter should apply
    ///                     changes to its internal data based on this output, then wrap it in a
    ///                     SimulationData variant.
    ///
    /// # Return Value:
    /// A variant of `SimulationData` containing the adapter's computed data after taking into account the
    /// given `engine_output` and the previous state of the adapter.
    fn to_simulation_data(
        self,
        prev_state: &SimulationData,
        engine_output: &EngineOutput,
    ) -> Result<SimulationData, Self::AdapterError>;
}

/// A backend component responsible for using [`SimulationDataAdapter`] to process the output of the
/// backend's engine.
///
/// It is very memory efficient, as it attempts to retrieve its adapter using the adapter's [`TryFrom`]
/// implementation and the [`SimulationData`] returned from the pool. Only if the pool doesn't have
/// an already-allocated `SimulationData` object matching the adapter's type, then the processor uses the
/// adapter initializer given to it.
///
/// The processor is also highly configurable, because if its user wanted to change the adapter type,
/// all they'd need is to call [`SimulationOutputProcessor::change_adapter`], and viola, it is changed!
/// The new adapter will use the same pool, so it is safe to call even when other threads hold senders
/// or [`Fish`] to the current pool.
///
/// **Note:** If the processor receives a `SimulationData` from the pool that fails to be converted to
///           the processor's adapter, the processor will **not** return it to the pool (in order to make
///           room for `SimulationData` objects which are valid for the adapter).
///           Due to this reason, **never** hand the same pool to two processors with different adapter
///           types.
pub struct SimulationOutputProcessor<A, I>
where
    A: SimulationDataAdapter,
    I: Fn() -> A,
{
    adapters_pool: Arc<Pool<SimulationData>>,
    adapter_initializer: I,
}

impl<A, I> SimulationOutputProcessor<A, I>
where
    A: SimulationDataAdapter,
    I: Fn() -> A,
{
    pub fn new(adapters_pool: Arc<Pool<SimulationData>>, initializer: I) -> Self {
        Self {
            adapters_pool,
            adapter_initializer: initializer,
        }
    }

    /// Retrieves a new adapter the caller may write to and/or consume.
    ///
    /// The function will first attempt to convert [`SimulationData`] from the processor's pool
    /// into an adapter. If the pool was empty, or every `SimulationData` object from it failed
    /// to be converted to an adapter, the function will use an initializer to create the adapter.
    fn get_writeable_adapter(&self) -> A {
        while let Some(allocated_data) = self.adapters_pool.try_get_fish().map(Fish::into_inner) {
            if let Ok(adapter) = A::try_from(allocated_data) {
                return adapter;
            }
            // TODO: Consider logging conversion errors here
        }
        (self.adapter_initializer)()
    }

    /// Processes output from the backend's engine, using the processor's unique adapter to apply
    /// the given [`EngineOutput`] to the previous state of the adapter.
    ///
    /// The processor will attempt to reuse already-allocated [`SimulationData`] objects in its pool,
    /// to avoid allocating new memory to write to.
    ///
    /// # Arguments:
    /// * `prev_state` - The previous state that the processor yielded. Note this has to match the
    ///                  current [`SimulationData`] variant that the current adapter returns, otherwise
    ///                  the function will fail.<br>
    ///                  **For Example -** If the current adapter type is something like `PressureDataAdapter`,
    ///                  but a non-pressure variant of `SimulationData` is returned, the
    ///                  function will return an [`Err`] variant.
    /// * `engine_output` - The output of the `engine` module. Its information will be used to change the
    ///                     previous state and compute new `SimulationData`.
    ///
    /// # Return Value:
    /// New [`SimulationData`] based on the previous adapter state and the new engine output.
    pub fn process_engine_output(
        &self,
        prev_state: &SimulationData,
        engine_output: &EngineOutput,
    ) -> Result<SimulationData, A::AdapterError> {
        self.get_writeable_adapter()
            .to_simulation_data(prev_state, engine_output)
    }

    /// Changes the adapter type of the processor.
    ///
    /// This function changes the adapter type yet keeps the original pool, making it safe to call even
    /// when other threads hold senders to the processor's pool.
    ///
    /// # Arguments:
    /// * `new_initializer` - An initializer to the new adapter type.
    ///
    /// # Type Arguments:
    /// * `NA` - The new adapter type of the processor.
    /// * `NI` - The new initializer's type.
    ///
    /// # Return Value:
    /// A `SimulationOutputProcessor` whose adapter's type is `NA`, yet uses the same [`Pool`] as the
    /// previous processor.
    pub fn change_adapter<NA, NI>(self, new_initializer: NI) -> SimulationOutputProcessor<NA, NI>
    where
        NA: SimulationDataAdapter,
        NI: Fn() -> NA,
    {
        SimulationOutputProcessor {
            adapters_pool: self.adapters_pool,
            adapter_initializer: new_initializer,
        }
    }
}
