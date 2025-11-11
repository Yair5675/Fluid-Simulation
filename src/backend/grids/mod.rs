//! A module defining grids in the fluid simulation - holding data in a 2-dimensional data
//! structure.

mod one_dimensional;
mod staggered;

// Expose grids directly to save users headache:
#[allow(unused_imports)]
pub use one_dimensional::OneDimensionalGrid;
#[allow(unused_imports)]
pub use staggered::StaggeredVelocityField;

/// Flexible trait for different grids in the simulation's backend.
pub trait Grid {
    /// The type of value the grid holds.
    type GridValue;

    /// Retrieves a reference to the `GridValue` located at `Grid(x, y)`.
    ///
    /// # Arguments:
    /// * `x` - Horizontal index in the grid.
    /// * `y` - Vertical index in the grid.
    ///
    /// # Return Value:
    /// A reference to the value at coordinates `x`, `y` in the grid, or `None` such coordinates
    /// point to outside the grid.
    fn get(&self, x: usize, y: usize) -> Option<&Self::GridValue>;

    /// Sets the `GridValue` located at `Grid(x, y)`.
    ///
    /// # Arguments:
    /// * `x` - Horizontal index in the grid.
    /// * `y` - Vertical index in the grid.
    /// * `new_value` - The new value which will be stored at the given position in the grid.
    ///
    /// # Return Value:
    /// `Some(())` if the new value was stored at the given coordinates, or `None` such coordinates
    /// point to outside the grid.
    fn set(&mut self, x: usize, y: usize, new_value: Self::GridValue) -> Option<()>;
}
