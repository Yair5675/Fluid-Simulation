//! This module is responsible for the heart of the fluid simulation - the velocity field
//! computation.

use std::{sync::Arc, time::Duration};

use anyhow::anyhow;
use vector2d::Vector2D;

use crate::backend::{
    grid::Grid,
    pool::{Fish, Pool},
};

// TODO: Add to some physics constants file / physics config:
const G: f64 = 9.81;

/// The final output of the [`SimulationEngine`] struct when it computes a single
/// simulation timestep.
///
/// It is expected to be an expensive struct to allocate, so only do so when necessary.
#[derive(Debug)]
pub struct EngineOutput {
    /// A grid of velocities that are stored at the **edges** of each cell.
    ///
    /// For a grid of width `w` and height `h` (meaning `w * h` cells), a staggered grid
    /// would require `w + 1` and `h + 1` width and height respectively.
    /// One could think about "shifting" the velocity grid by half a cell diagonally, then
    /// separating the vertical and horizontal components.
    ///
    /// Example of a staggered grid's cell vs regular velocity grid cell:
    /// <figure>
    ///
    ///            Staggered Grid                  Regular grid:
    ///                  /\
    ///         +--------||--------+            +------------------+
    ///         |                  |            |        __ .      |
    ///         |                  |            |          / \     |
    ///        ===>              <===           |         /        |
    ///         |                  |            |                  |
    ///         |                  |            |                  |
    ///         +--------||--------+            +------------------+
    ///                  \/
    /// </figure>
    ///
    /// To find the specific velocity component of a cell at position `(x, y)`, use
    /// this reference:
    /// * **top** - Position `(x, y)`, vertical component.
    /// * **bottom** - Position `(x, y + 1)`, vertical component.
    /// * **left** - Position `(x, y)`, horizontal component.
    /// * **right** - Position `(x + 1, y)`, horizontal component.
    ///
    /// Hopefully you can see from these calculations why the width and height had to be increased by 1.
    pub staggered_velocities: Grid<Vector2D<f64>>,
} // TODO: Declare fields as necessary

impl EngineOutput {
    /// Initializes a new `EngineOutput` object.
    ///
    /// This operation is considered expensive, and should only be done if the caller
    /// has no access to an already allocated `EngineOutput`.
    ///
    /// `EngineOutput` contains a "staggered grid" of velocities, which means it stores
    /// velocities at the edges of cells instead of at their centers.
    /// Due to this data type, the width and height used will be the given `width` and
    /// `height` arguments, plus 1.
    ///
    /// Users should not bear that in mind when passing arguments, the function will take
    /// care of it.
    ///
    /// # Arguments:
    /// * `width` - Number of cells in the horizontal axis of the outputted grid.
    /// * `height` - Number of cells in the vertical axis of the outputted grid.
    ///
    /// # Return Value:
    /// A new `EngineOutput` object.
    pub fn new(width: usize, height: usize) -> Self {
        let staggered_velocities = Grid::new(width + 1, height + 1);
        Self {
            staggered_velocities,
        }
    }
}

/// The engine that computes the fluid simulation's
pub struct SimulationEngine {
    engine_output_pool: Arc<Pool<EngineOutput>>,
    // TODO: Put in some kind of configuration in the final version:
    grid_width: usize,
    grid_height: usize,
}

impl SimulationEngine {
    /// Creates a new simulation engine that employs efficient memory reuse by using the given pool.
    ///
    /// # Arguments:
    /// * `grid_width` - Number of cells in the horizontal axis of the outputted grid.
    /// * `grid_height` - Number of cells in the vertical axis of the outputted grid.
    /// * `pool` - A pool of [`EngineOutput`] objects, wrapped in an `Arc` so that other threads
    ///            can populate it as well.
    ///            The pool doesn't have to contain any values when passed to the function. The
    ///            engine will allocate a new object if the pool is empty.
    ///
    /// # Return Value:
    /// A new `SimulationEngine` object that attempts to fish from the given pool.
    pub fn new(grid_width: usize, grid_height: usize, pool: Arc<Pool<EngineOutput>>) -> Self {
        Self {
            grid_width,
            grid_height,
            engine_output_pool: pool,
        }
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
            self.engine_output_pool
                .get_fish_or_init(|| EngineOutput::new(self.grid_width, self.grid_height))
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
    /// them. If any error occurred during the computation, it is returned (although it should be pretty unlikely).
    pub fn compute_timestep(
        &self,
        dt: Duration,
        prev_timestep: &EngineOutput,
        wait_for_pool: bool,
    ) -> anyhow::Result<Fish<EngineOutput>> {
        let mut output = self.get_engine_output(wait_for_pool);
        self.compute_timestep_internal(dt, prev_timestep, &mut output)?;
        Ok(output)
    }

    /// The actual physics of the engine.
    fn compute_timestep_internal(
        &self,
        dt: Duration,
        prev_timestep: &EngineOutput,
        output_buffer: &mut EngineOutput,
    ) -> anyhow::Result<()> {
        todo!("Implement actual physics here!")
    }

    /// Applies gravity to every cell's vertical component, except for the ceiling and the floor.
    fn apply_gravity(
        &self,
        dt: &Duration,
        prev_timestep: &EngineOutput,
        output_buffer: &mut EngineOutput,
    ) -> anyhow::Result<()> {
        let gravity = Vector2D::new(0.0, -G);

        // Stop before final horizontal component since that column is only relevant for horizontal
        // components:
        for x in 0..self.grid_width {
            // Start from 1 and stop before final vertical component to skip topmost and lowest
            // components:
            for y in 1..self.grid_height {
                let prev_velocity =
                    prev_timestep
                        .staggered_velocities
                        .get(x, y)
                        .ok_or_else(|| {
                            anyhow!("Previous timestep's dimensions are different from engine's")
                                .context(format!("Missing cell: ({}, {})", x, y))
                        })?;

                output_buffer
                    .staggered_velocities
                    .set(x, y, *prev_velocity + (gravity * dt.as_secs_f64()))
                    .ok_or_else(|| {
                        anyhow!("Output buffer's dimensions are different from engine's")
                            .context(format!("Missing cell: ({}, {})", x, y))
                    })?;
            }
        }

        Ok(())
    }
}
