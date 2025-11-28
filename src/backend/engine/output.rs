//! Module holding the data types computed by the backend engine.

use vector2d::Vector2D;

use crate::backend::grid::Grid;

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
    ///
    /// Since each edge (except for the walls) is shared between two cells, the velocities cannot be saved
    /// relative to the center of the cell they are near (i.e - a positive value cannot indicate "going out
    /// of the cell", since the velocity in this case would go INTO the cell next to the current one).<br>
    /// Therefor, the sign of a velocity component is defined as such:
    /// * **Positive Vertical** - Downwards velocity.
    /// * **Negative Vertical** - Upwards velocity.
    /// * **Positive Horizontal** - Rightwards velocity.
    /// * **Negative Horizontal** - Leftwards velocity.
    ///
    /// This definition aligns with the indexing of velocities, so it should be pretty clear.
    pub staggered_velocities: Grid<Vector2D<f64>>,
}

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
