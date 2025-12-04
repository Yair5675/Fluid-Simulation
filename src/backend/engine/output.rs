//! Module holding the data types computed by the backend engine.

use vector2d::Vector2D;

/// A simple struct representing a fluid particle, with position and velocity.
/// 
/// Note the axes of the particle's position and velocity are in accordance with the staggered
/// grid:
/// * **Positive Vertical** - Down.
/// * **Negative Vertical** - Up.
/// * **Positive Horizontal** - Right.
/// * **Negative Horizontal** - Left.
#[derive(Debug, Clone, Copy)]
pub struct Particle {
    pub pos: Vector2D<f64>,
    pub vel: Vector2D<f64>,
}

/// The final output of the [`SimulationEngine`] struct when it computes a single
/// simulation timestep.
///
/// It is expected to be an expensive struct to allocate, so only do so when necessary.
#[derive(Debug)]
pub struct EngineOutput {
    pub grid_width: usize,
    pub grid_height: usize,
    pub particles: Vec<Particle>,
}

impl EngineOutput {
    /// Initializes a new `EngineOutput` object.
    ///
    /// This operation is considered expensive, and should only be done if the caller
    /// has no access to an already allocated `EngineOutput`.
    ///
    /// # Arguments:
    /// * `particles_count` - Number of particle the simulation wants to compute.
    /// * `width` - Number of cells in the horizontal axis of the grid used to store velocities.
    /// * `height` - Number of cells in the vertical axis of the grid used to store velocities.
    ///
    /// # Return Value:
    /// A new `EngineOutput` object.
    pub fn new(mut particles_count: usize, width: usize, height: usize) -> Self {
        // TODO: Handle cases where width or height are less than/equal to 2
        let mut particles = Vec::with_capacity(particles_count);
        let particles_per_cell =
            (((width - 2) * (height - 2)) as f64 / particles_count as f64).ceil();

        let distance_from_wall = 0.01;
        let default_vel = Vector2D::new(0.0, 0.0);

        for x in 1..(width - 1) {
            for y in 1..(height - 1) {
                let particles_in_this_cell = particles_per_cell.min(particles_count as f64);
                let x_increment = (particles_in_this_cell + 1.0).recip();
                for p in 0..(particles_in_this_cell as usize) {
                    let pos = Vector2D {
                        x: distance_from_wall + x as f64 + p as f64 * x_increment,
                        y: y as f64 + distance_from_wall,
                    };
                    particles.push(Particle {
                        pos,
                        vel: default_vel,
                    });
                }
            }
        }

        Self {
            grid_width: width,
            grid_height: height,
            particles,
        }
    }
}
