//! This module is responsible for the heart of the fluid simulation - the velocity field
//! computation.

// A grid of velocities that are stored at the **edges** of each cell.
//
// For a grid of width `w` and height `h` (meaning `w * h` cells), a staggered grid
// would require `w + 1` and `h + 1` width and height respectively.
// One could think about "shifting" the velocity grid by half a cell diagonally, then
// separating the vertical and horizontal components.
//
// Example of a staggered grid's cell vs regular velocity grid cell:
// <figure>
//
//            Staggered Grid                  Regular grid:
//                  /\
//         +--------||--------+            +------------------+
//         |                  |            |        __ .      |
//         |                  |            |          / \     |
//        ===>              <===           |         /        |
//         |                  |            |                  |
//         |                  |            |                  |
//         +--------||--------+            +------------------+
//                  \/
// </figure>
//
// To find the specific velocity component of a cell at position `(x, y)`, use
// this reference:
// * **top** - Position `(x, y)`, vertical component.
// * **bottom** - Position `(x, y + 1)`, vertical component.
// * **left** - Position `(x, y)`, horizontal component.
// * **right** - Position `(x + 1, y)`, horizontal component.
//
// Hopefully you can see from these calculations why the width and height had to be increased by 1.
//
// Since each edge (except for the walls) is shared between two cells, the velocities cannot be saved
// relative to the center of the cell they are near (i.e - a positive value cannot indicate "going out
// of the cell", since the velocity in this case would go INTO the cell next to the current one).<br>
// Therefor, the sign of a velocity component is defined as such:
// * **Positive Vertical** - Downwards velocity.
// * **Negative Vertical** - Upwards velocity.
// * **Positive Horizontal** - Rightwards velocity.
// * **Negative Horizontal** - Leftwards velocity.
//
// This definition aligns with the indexing of velocities, so it should be pretty clear.

use std::{sync::Arc, time::Duration};

use anyhow::ensure;
use vector2d::Vector2D;

use crate::backend::{
    engine::output::Particle,
    grid::Grid,
    pool::{Fish, Pool},
};

// TODO: Add to some physics constants file / physics config:
const G: f64 = 9.81;
const DEFAULT_PROJECTIONS_ITERATIONS: usize = 25;
const DEFAULT_OVERRELAXATION_FACTOR: f64 = 1.9;
const DEFAULT_VELOCITY_ABSORPTION_FACTOR: f64 = 0.3;
const DEFAULT_GRID_SPACING: f64 = 1.0;
const DEFAULT_STIFFNESS_FACTOR: f64 = 1.0;
const DEFAULT_PIC_FACTOR: f64 = 0.1;
const DEFAULT_FLIP_FACTOR: f64 = 1.0 - DEFAULT_PIC_FACTOR;

mod output;

pub use output::EngineOutput;

/// The state of a given state in terms of material.
#[derive(Debug, Clone, Copy)]
enum CellState {
    /// A solid cell, can represent the boundaries of the grid to prevent it from escaping, or an obstacle.
    /// ![](https://media.tenor.com/FfNjau1IYxMAAAAe/solidsnake-meme.png)
    Solid,
    /// A cell full of water.
    Water,
    /// A cell full of air.
    Air,
}

impl CellState {
    /// Each cell's velocity should be changed differently based on its type.
    /// This function returns a weight between 0.0 and 1.0 that can be multiplied by the velocity correction
    /// to this cell to yield the disered correction.
    ///
    /// Solid state for example yields a weight of 0.0, because they should not be affected, while water and air
    /// cells yield 1.0.
    pub fn velocity_correction_weight(&self) -> f64 {
        match self {
            CellState::Solid => 0.0,
            CellState::Air => 1.0,
            CellState::Water => 1.0,
        }
    }
}

/// Represents any set of 4 weights of a cell. Honestly it's just a wrapper of 4 floats
/// that saves documentation.
struct CellWeights {
    pub topleft: f64,
    pub topright: f64,
    pub bottomleft: f64,
    pub bottomright: f64,
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
    /// Staggered grid of velocity weights, calculated from all particles' positions relative to the cell they're in.
    staggered_weights: Grid<Vector2D<f64>>,
    grid_state: Grid<CellState>, // TODO - move to main backend struct and accept as parameter here, to allow the
    //        adapters to read from the state too, and handle frontend messages somewhere
    //        else.
    // TODO: Put in some kind of configuration in the final version:
    grid_width: usize,
    grid_height: usize,
    grid_spacing: f64,
    particles_count: usize,
    projection_iterations: usize,
    overrelaxation_factor: f64,
    velocity_absorption_factor: f64,
    stiffness_factor: f64,
    pic_factor: f64,
    flip_factor: f64,
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
    pub fn new(
        particles_count: usize,
        grid_width: usize,
        grid_height: usize,
        pool: Arc<Pool<EngineOutput>>,
    ) -> Self {
        let staggered_velocities = (
            Grid::new(grid_width + 1, grid_height + 1),
            Grid::new(grid_width + 1, grid_height + 1),
        );

        Self {
            grid_width,
            grid_height,
            particles_count,
            staggered_velocities,
            staggered_weights: Grid::new(grid_width + 1, grid_height + 1),
            grid_spacing: DEFAULT_GRID_SPACING,
            projection_iterations: DEFAULT_PROJECTIONS_ITERATIONS,
            overrelaxation_factor: DEFAULT_OVERRELAXATION_FACTOR,
            velocity_absorption_factor: DEFAULT_VELOCITY_ABSORPTION_FACTOR,
            stiffness_factor: DEFAULT_STIFFNESS_FACTOR,
            pic_factor: DEFAULT_PIC_FACTOR,
            flip_factor: DEFAULT_FLIP_FACTOR,
            grid_state: Self::build_initial_state_grid(grid_width, grid_height),
            engine_output_pool: pool,
        }
    }

    fn build_initial_state_grid(grid_width: usize, grid_height: usize) -> Grid<CellState> {
        let mut grid_state = Vec::with_capacity(grid_height);

        // Top row:
        grid_state.push(vec![CellState::Solid; grid_width]);

        // Interior rows:
        for _ in 0..(grid_height - 2) {
            let mut row = Vec::with_capacity(grid_width);
            row.push(CellState::Solid);
            row.extend(std::iter::repeat(CellState::Air).take(grid_width - 2));
            row.push(CellState::Solid);
            grid_state.push(row);
        }

        // Bottom row
        grid_state.push(vec![CellState::Solid; grid_width]);

        Grid::from(grid_state)
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
            self.engine_output_pool.get_fish_or_init(|| {
                EngineOutput::new(
                    self.grid_spacing,
                    self.particles_count,
                    self.grid_width,
                    self.grid_height,
                )
            })
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
        &mut self,
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
        &mut self,
        dt: Duration,
        prev_timestep: &EngineOutput,
        output_buffer: &mut EngineOutput,
    ) -> anyhow::Result<()> {
        ensure!(
            output_buffer.grid_width == self.grid_width && output_buffer.grid_height == self.grid_height,
            format!("Output buffer doesn't have engine's dimensions (output=[{}, {}], engine=[{}, {}])",
             output_buffer.grid_width, output_buffer.grid_height, self.grid_width, self.grid_height)
        );

        self.apply_forces_and_update_state(&dt, prev_timestep, output_buffer);
        self.transfer_particles_to_grids(&output_buffer.particles);
        self.apply_projection(prev_timestep.rest_density, &output_buffer.densities);
        self.transfer_grids_to_particles(&mut output_buffer.particles);

        Ok(())
    }

    /// Applies forces to the particles, moves them, updates the state grid and the densities grid in the output buffer.
    fn apply_forces_and_update_state(
        &mut self,
        dt: &Duration,
        prev_timestep: &EngineOutput,
        output_buffer: &mut EngineOutput,
    ) {
        // Clear state for any previously-water cell
        self.grid_state.for_each_mut(|state, _| {
            if let &mut CellState::Water = state {
                *state = CellState::Air;
            }
        });

        // Clear density:
        output_buffer.densities.set_all_with(Default::default);

        prev_timestep
            .particles
            .iter()
            .zip(output_buffer.particles.iter_mut())
            .for_each(|(in_particle, out)| {
                self.simulate_particle_movement(dt, in_particle, out);

                // Mark the cell as water (it has a water particle):
                let coords = (
                    (out.pos.x / self.grid_spacing) as usize,
                    (out.pos.y / self.grid_spacing) as usize,
                );
                self.grid_state.set(coords.0, coords.1, CellState::Water);

                // Add scaled density:
                output_buffer
                    .densities
                    .get_mut(coords.0, coords.1)
                    .map(|density| {
                        *density += (self.particles_count as f64).recip();
                    });
            });
    }

    /// Simulates the 2-D particle's movement in space - applying acceleration and velocity to its velocity
    /// and position respectively.
    ///
    /// If the new position is inside a solid cell, the function moves the particle out of the way.
    ///
    /// **Note**: The function assumes `in_particle` was not in a solid cell.
    fn simulate_particle_movement(
        &self,
        dt: &Duration,
        in_particle: &Particle,
        out_particle: &mut Particle,
    ) {
        let dt_secs = dt.as_secs_f64();
        out_particle.vel.y = in_particle.vel.y + G * dt_secs;
        out_particle.vel.x = in_particle.vel.x;

        out_particle.pos.x = in_particle.pos.x + dt_secs * out_particle.vel.x;
        out_particle.pos.y = in_particle.pos.y + dt_secs * out_particle.vel.y;

        // If we hit a solid, some energy is absorbed and the velocity is inverted:
        if let Some(CellState::Solid) | None = self.get_cell_by_position(&out_particle.pos) {
            let energy_remaining = 1. - self.velocity_absorption_factor;
            out_particle.pos = in_particle.pos;
            out_particle.vel.x *= -energy_remaining;
            out_particle.vel.y *= -energy_remaining;
        }
    }

    fn get_cell_by_position(&self, pos: &Vector2D<f64>) -> Option<&CellState> {
        self.grid_state.get(
            (pos.x / self.grid_spacing) as usize,
            (pos.y / self.grid_spacing) as usize,
        )
    }

    /// Transfers the velocity of each particle to the two staggered velocity grids in the engine.
    ///
    /// The velocity magnitude is divided between the edges of the cell the particle is in according to a
    /// a bilinear interpolation depending on the particle position's distance from the cell's topleft corner.
    fn transfer_particles_to_grids(&mut self, particles: &Vec<Particle>) {
        // Clear staggered weights and velocities:
        self.staggered_weights.set_all_with(Default::default);
        self.staggered_velocities.0.set_all_with(Default::default);

        for particle in particles.iter() {
            self.transfer_particle_velocity_to_staggered_grids(particle, true);
            self.transfer_particle_velocity_to_staggered_grids(particle, false);
        }
    }

    fn transfer_particle_velocity_to_staggered_grids(&mut self, particle: &Particle, is_horizontal: bool) {
        // Adjust particle position to the staggerd grid (staggered grid is like a normal grid shifted by half
        // a cell, so we need to shift the particle down by half a cell):
        let staggered_pos = if is_horizontal {
            (particle.pos.x, particle.pos.y + self.grid_spacing * 0.5)
        } else {
            (particle.pos.x + self.grid_spacing * 0.5, particle.pos.y)
        };

        let coords = (
            (staggered_pos.0 / self.grid_spacing) as usize,
            (staggered_pos.1 / self.grid_spacing) as usize,
        );

        // Transfer velocity to the first grid:
        let weights = self.calculate_bilinear_weights(&staggered_pos, &coords);
        let velocity = if is_horizontal {
            particle.vel.x
        } else {
            particle.vel.y
        };
        self.add_weighted_velocity(coords, velocity, weights, is_horizontal);

        // Scale down the velocities by the sum of weights, and copy to the second grid on the way:
        self.scale_down_staggered_velocities(is_horizontal);
    }

    fn scale_down_staggered_velocities(&mut self, is_horizontal: bool) {
        for x in 1..(self.grid_width - 1) {
            for y in 1..(self.grid_height - 1) {
                if !self.is_fluid_cell(x, y) {
                    continue;
                }

                // is_fluid_cell guarantees we are in a valid coordinate
                unsafe {
                    let weights_sum = self.staggered_weights.get_unchecked(x, y);
                    let current_velocity = self.staggered_velocities.0.get_unchecked_mut(x, y);
                    if is_horizontal {
                        current_velocity.x /= weights_sum.x;
                    } else {
                        current_velocity.y /= weights_sum.y;
                    }

                    // Copy final result to second velocity grid:
                    self.staggered_velocities.1.get_unchecked_mut(x, y).x = current_velocity.x;
                }
            }
        }
    }

    fn calculate_bilinear_weights(
        &self,
        staggered_pos: &(f64, f64),
        coords: &(usize, usize),
    ) -> CellWeights {
        // Compute scaled deltas once (minor optimization):
        let scaled_deltas = (
            (staggered_pos.0 - (coords.0 as f64 * self.grid_spacing)) / self.grid_spacing,
            (staggered_pos.1 - (coords.1 as f64 * self.grid_spacing)) / self.grid_spacing,
        );

        CellWeights {
            topleft: (1.0 - scaled_deltas.0) * (1.0 - scaled_deltas.1),
            topright: scaled_deltas.0 * (1.0 - scaled_deltas.1),
            bottomleft: (1.0 - scaled_deltas.0) * scaled_deltas.1,
            bottomright: scaled_deltas.0 * scaled_deltas.1,
        }
    }

    /// Given a velocity magnitude and a set of weights referring to how the velocity should be spread in a cell,
    /// the function adds the weighted velocity to the staggered velocity and weight grids.
    ///
    /// The function will only add the weighted velocity to water cells and will skip air/solid cells.
    ///
    /// **NOTE** - The function only affects `self.staggered_velocities.0`.
    ///
    /// # Arguments:
    /// * `cell_coords` - integer coordinates of the cell the velocity is in.
    /// * `velocity` - Magnitude of the velocity of some particle in the cell specified.
    /// * `weights` - A set of 4 weights, determining how much of `velocity` will be transfered to the cell's edges.
    fn add_weighted_velocity(
        &mut self,
        cell_coords: (usize, usize),
        velocity: f64,
        weights: CellWeights,
        is_horizontal: bool,
    ) {
        let coords_weights_array = [
            ((cell_coords.0, cell_coords.1), weights.topleft),
            ((cell_coords.0 + 1, cell_coords.1), weights.topright),
            ((cell_coords.0, cell_coords.1 + 1), weights.bottomleft),
            ((cell_coords.0 + 1, cell_coords.1 + 1), weights.bottomright),
        ];

        for (coords, weight) in coords_weights_array.into_iter() {
            if self.is_fluid_cell(coords.0, coords.1) {
                // If is_fluid_cell is true, the coordinates are guaranteed to exist:
                unsafe {
                    let velocity_cell = self.staggered_velocities
                        .0
                        .get_unchecked_mut(coords.0, coords.1);
                    if is_horizontal {
                        velocity_cell.x += velocity * weight;
                        self.staggered_weights.get_unchecked_mut(coords.0, coords.1).x += weight;
                    } else {
                        velocity_cell.y += velocity * weight;
                        self.staggered_weights.get_unchecked_mut(coords.0, coords.1).y += weight;
                    }
                }
            }
        }
    }

    fn is_fluid_cell(&self, x: usize, y: usize) -> bool {
        if let Some(&CellState::Water) = self.grid_state.get(x, y) {
            return true;
        }
        false
    }

    /// Applies the projection step of the simulation (i.e - ensuring incompressibility).
    fn apply_projection(&mut self, rest_density: f64, densities: &Grid<f64>) {
        // TODO: Ensure densities has same dimensions as all other grids
        for _ in 0..self.projection_iterations {
            // Start from 1 to skip left wall, and stop before the right wall:
            for x in 1..(self.grid_width - 1) {
                // Start from 1 to skip ceiling, and stop before the floor:
                for y in 1..(self.grid_height - 1) {
                    unsafe {
                        let divergence = self.overrelaxation_factor
                            * Self::unchecked_calculate_divergence(
                                x,
                                y,
                                &self.staggered_velocities.1,
                            )
                            - self.stiffness_factor
                                * (densities.get_unchecked(x, y) - rest_density); // Causes more outward push in dense regions
                        let fluid_neighbors = self.count_fluid_neighbors(x, y);
                        let velocity_correction = divergence / fluid_neighbors as f64;

                        // Multiply by state since solid is 0 (and will not affect anything), while fluid is 1:
                        let topleft = self.staggered_velocities.1.get_unchecked_mut(x, y);
                        topleft.x += velocity_correction
                            * self
                                .grid_state
                                .get_unchecked(x - 1, y)
                                .velocity_correction_weight();
                        topleft.y += velocity_correction
                            * self
                                .grid_state
                                .get_unchecked(x, y - 1)
                                .velocity_correction_weight();

                        let bottom = self.staggered_velocities.1.get_unchecked_mut(x, y + 1);
                        bottom.y += velocity_correction
                            * self
                                .grid_state
                                .get_unchecked(x, y + 1)
                                .velocity_correction_weight();

                        let right = self.staggered_velocities.1.get_unchecked_mut(x + 1, y);
                        right.x += velocity_correction
                            * self
                                .grid_state
                                .get_unchecked(x + 1, y)
                                .velocity_correction_weight();
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

    fn transfer_grids_to_particles(&self, particles: &mut Vec<Particle>) {
        for particle in particles.iter_mut() {
            let pre_projection = self.calculate_velocity_from_grid(&self.staggered_velocities.0, &particle.pos);
            let post_projection = self.calculate_velocity_from_grid(&self.staggered_velocities.1, &particle.pos);

            let pic = post_projection;
            let flip = post_projection - pre_projection;

            particle.vel = pic * self.pic_factor + flip * self.flip_factor;
        }
    }

    fn calculate_velocity_from_grid(&self, staggered_velocities: &Grid<Vector2D<f64>>, pos: &Vector2D<f64>) -> Vector2D<f64> {
        let coords = (
            (pos.x / self.grid_spacing) as usize,
            (pos.y / self.grid_spacing) as usize,
        );
        // Topleft, topright, bottomleft, bottomright:
        let coords = [
            (coords.0, coords.1),
            (coords.0 + 1, coords.1),
            (coords.0, coords.1 + 1),
            (coords.0 + 1, coords.1 + 1)
        ];
        
        let mut new_velocity = Vector2D::new(0.0, 0.0);
        let mut weights_sum = Vector2D::new(0.0, 0.0);

        for (x, y) in coords.into_iter() {
            if self.is_fluid_cell(x, y) {
                // is_fluid_cell guarantees the cell exists in the grids
                unsafe {
                    let weight = self.staggered_weights.get_unchecked(x, y);
                    let velocity = staggered_velocities.get_unchecked(x, y);

                    new_velocity.x += weight.x * velocity.x;
                    new_velocity.y += weight.y * velocity.y;

                    weights_sum.x += weight.x;
                    weights_sum.y += weight.y;
                }
            }
        }

        if weights_sum.x != 0.0 {
            new_velocity.x /= weights_sum.x;
        }
        if weights_sum.y != 0.0 {
            new_velocity.y /= weights_sum.y;
        }

        new_velocity
    }

    fn do_grid_dimensions_match<C>(&self, grid: &Grid<C>) -> bool {
        self.grid_width == grid.width() && self.grid_height == grid.height()
    }

    fn do_staggered_grid_dimensions_match<C>(&self, staggered_grid: &Grid<C>) -> bool {
        self.grid_width + 1 == staggered_grid.width()
            && self.grid_height + 1 == staggered_grid.height()
    }
}
