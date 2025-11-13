//! Module defining a special kind of grid which hold velocities at the EDGES of each cell - the staggered
//! grid.

use vector2d::Vector2D;
use crate::backend::grids::OneDimensionalGrid;

/// A special type of grid which holds velocity values at the center of each cell's edges.
pub struct StaggeredVelocityField {
    /// The actual data-structure storing the staggered velocities
    raw_grid: OneDimensionalGrid<Vector2D<f64>>, // Implementation detail - OneDimensionalGrid used (any Grid will do).
}

impl StaggeredVelocityField {
    /// Creates a new `StaggeredVelocityField` where all of the stored velocity vectors are set to
    /// `[ 0.0, 0.0 ]`.
    ///
    /// # Arguments:
    /// * `width` - The horizontal length of the grid.
    /// * `height` - The vertical length of the grid.
    ///
    /// # Return Value:
    /// A staggered velocity field whose velocities are `[ 0.0, 0.0 ]`.
    pub fn new(width: usize, height: usize) -> Self {
        // Since we have 4 values per cell instead of 2, we need to add 1 row and 1 column.
        let raw_grid = OneDimensionalGrid::new(width + 1, height + 1);
        Self { raw_grid }
    }
}
