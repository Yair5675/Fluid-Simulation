//! This module is responsible for the heart of the fluid simulation - the velocity field
//! computation.

use std::{sync::Arc, time::Duration};

use anyhow::{anyhow, ensure};
use vector2d::Vector2D;

use crate::backend::{
    grid::Grid,
    pool::{Fish, Pool},
};

// TODO: Add to some physics constants file / physics config:
const G: f64 = 9.81;
const DEFAULT_PROJECTIONS_ITERATIONS: usize = 25;
const DEFAULT_OVERRELAXATION_FACTOR: f64 = 1.9;

mod output;

pub use output::EngineOutput;

/// The state of a given state in terms of material.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum CellState {
    /// A solid cell, can represent the boundaries of the grid to prevent it from escaping, or an obstacle.
    /// ![](https://media.tenor.com/FfNjau1IYxMAAAAe/solidsnake-meme.png)
    Solid = 0,
    /// A cell full of some fluid (can be an actual fluid or gas).
    Fluid = 1,
}

/// The engine that computes the fluid simulation's
pub struct SimulationEngine {
    engine_output_pool: Arc<Pool<EngineOutput>>,
    /// In the PIC/FLIP method, the engine needs two separate staggered grids to hold the velocities:
    /// * One right *before* making the grid incompressible.
    /// * Another right *after* making the grid incompressible.
    /// In the final stage, the engine adds the difference between the grids back to the particle.
    /// 
    /// To avoid making a copy every single timestep, two are saved here, and will be modified over and over.
    /// The first grid will hold the not-yet-incompressible velocities, the second one will hold the already-incompressible
    /// velocities.
    staggered_velocities: (Grid<Vector2D<f64>>, Grid<Vector2D<f64>>),
    grid_state: Grid<CellState>, // TODO - move to main backend struct and accept as parameter here, to allow the
    //        adapters to read from the state too, and handle frontend messages somewhere
    //        else.
    // TODO: Put in some kind of configuration in the final version:
    grid_width: usize,
    grid_height: usize,
    projection_iterations: usize,
    overrelaxation_factor: f64,
}

impl SimulationEngine {
    /// Creates a new simulation engine that employs efficient memory reuse by using the given pool.
    /// The engine will initialize its cells' state to a grid full of fluid whose boundaries are solid.
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
        let staggered_velocities = (
            Grid::new(grid_width + 1, grid_height + 1),
            Grid::new(grid_width + 1, grid_height + 1)
        );
        let mut grid_state = Vec::with_capacity(grid_height);

        // Top row:
        grid_state.push(vec![CellState::Solid; grid_width]);

        // Interior rows:
        for _ in 0..(grid_height - 2) {
            let mut row = Vec::with_capacity(grid_width);
            row.push(CellState::Solid);
            row.extend(std::iter::repeat(CellState::Fluid).take(grid_width - 2));
            row.push(CellState::Solid);
            grid_state.push(row);
        }

        // Bottom row
        grid_state.push(vec![CellState::Solid; grid_width]);

        Self {
            grid_width,
            grid_height,
            staggered_velocities,
            projection_iterations: DEFAULT_PROJECTIONS_ITERATIONS,
            overrelaxation_factor: DEFAULT_OVERRELAXATION_FACTOR,
            grid_state: Grid::from(grid_state),
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
        // Ensure all grids are synchronized in dimensions:
        ensure!(
            self.do_staggered_grid_dimensions_match(&prev_timestep.staggered_velocities),
            "Previous timestep's dimensions differ from engine's"
        );
        ensure!(
            self.do_staggered_grid_dimensions_match(&output_buffer.staggered_velocities),
            "Output buffer's dimensions differ from engine's"
        );
        ensure!(
            self.do_grid_dimensions_match(&self.grid_state),
            "State grid's dimensions differ from engine's"
        );

        todo!("Implement actual physics here!")
    }

    /// Applies gravity to every cell's vertical component, except for the ceiling and the floor.
    ///
    /// Note the function expects `prev_timestep`, `output_buffer` and `self.grid_state` to have dimensions
    /// equal to `self.grid_width` and `self.grid_height`.
    fn apply_gravity(
        &self,
        dt: &Duration,
        prev_timestep: &EngineOutput,
        output_buffer: &mut EngineOutput,
    ) {
        let gravity = Vector2D::new(0.0, -G);

        // Start from 1 to skip left wall, and stop before the right wall:
        for x in 1..(self.grid_width - 1) {
            // Start from 2 to skip the ceiling's top + bottom velocity component and stop before
            // the floor's top component:
            for y in 2..(self.grid_height - 1) {
                let state = self
                    .grid_state
                    .get(x, y)
                    .expect("Invariant broke - grid_state dimensions != engine's dimensions");
                if let &CellState::Solid = state {
                    continue;
                }

                let prev_velocity = prev_timestep
                    .staggered_velocities
                    .get(x, y)
                    .expect("Invariant broke - prev_timestep dimensions != engine's dimensions");

                output_buffer
                    .staggered_velocities
                    .set(x, y, *prev_velocity + (gravity * dt.as_secs_f64()))
                    .expect("Invariant broke - output_buffer dimensions != engine's dimensions");
            }
        }
    }

    /// Applies the projection step of the simulation (i.e - ensuring incompressibility).
    fn apply_projection(&self, current_timestep: &mut EngineOutput) {
        for _ in 0..self.projection_iterations {
            // Start from 1 to skip left wall, and stop before the right wall:
            for x in 1..(self.grid_width - 1) {
                // Start from 1 to skip ceiling, and stop before the floor:
                for y in 1..(self.grid_height - 1) {
                    unsafe {
                        let divergence = self.overrelaxation_factor * Self::unchecked_calculate_divergence(x, y, &current_timestep.staggered_velocities);
                        let fluid_neighbors = self.count_fluid_neighbors(x, y);
                        let velocity_correction = divergence / fluid_neighbors as f64;
                    
                        // Multiply by state since solid is 0 (and will not affect anything), while fluid is 1:
                        let topleft = current_timestep.staggered_velocities.get_unchecked_mut(x, y);
                        topleft.x += velocity_correction * ((*self.grid_state.get_unchecked(x - 1, y) as u8) as f64);
                        topleft.y += velocity_correction * ((*self.grid_state.get_unchecked(x, y - 1) as u8) as f64);

                        let bottom = current_timestep.staggered_velocities.get_unchecked_mut(x, y + 1);
                        bottom.y += velocity_correction * ((*self.grid_state.get_unchecked(x, y + 1) as u8) as f64);

                        let right = current_timestep.staggered_velocities.get_unchecked_mut(x + 1, y);
                        right.x += velocity_correction * ((*self.grid_state.get_unchecked(x + 1, y) as u8) as f64);
                    }
                    
                }
            }
        }
    }

    /// Calculates the divergence (total outflow) for a given cell in a staggered grid.
    ///
    /// The function assumes `x` and `y` are valid coordinates within the grid to increase
    /// performance. If they are outside the grid, expect undefined behavior.
    unsafe fn unchecked_calculate_divergence(
        x: usize,
        y: usize,
        staggered_velocities: &Grid<Vector2D<f64>>,
    ) -> f64 {
        unsafe {
            let top_outflow = -staggered_velocities.get(x, y).unwrap_unchecked().y;
            let bottom_outflow = staggered_velocities.get(x, y + 1).unwrap_unchecked().y;
            let left_outflow = -staggered_velocities.get(x, y).unwrap_unchecked().x;
            let right_outflow = staggered_velocities.get(x + 1, y).unwrap_unchecked().x;
            top_outflow + bottom_outflow + left_outflow + right_outflow
        }
    }

    fn count_fluid_neighbors(&self, x: usize, y: usize) -> u8 {
        let left_neighbor = self
            .grid_state
            .get(x - 1, y)
            .map(|&state| state as u8)
            .unwrap_or_default();
        let right_neighbor = self
            .grid_state
            .get(x + 1, y)
            .map(|&state| state as u8)
            .unwrap_or_default();
        let top_neighbor = self
            .grid_state
            .get(x, y - 1)
            .map(|&state| state as u8)
            .unwrap_or_default();
        let bottom_neighbor = self
            .grid_state
            .get(x, y + 1)
            .map(|&state| state as u8)
            .unwrap_or_default();

        left_neighbor + right_neighbor + top_neighbor + bottom_neighbor
    }

    fn do_grid_dimensions_match<C>(&self, grid: &Grid<C>) -> bool {
        self.grid_width == grid.width() && self.grid_height == grid.height()
    }

    fn do_staggered_grid_dimensions_match<C>(&self, staggered_grid: &Grid<C>) -> bool {
        self.grid_width + 1 == staggered_grid.width() && self.grid_height + 1 == staggered_grid.height()
    }
}
