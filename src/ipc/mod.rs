//! Responsible for containing data types, structs and traits which define and abstract the
//! communication between the frontend module and the backend module.
//!
//! This isn't technically an IPC since both run on the same process, but represents the same idea.

/// Represents a grid of computed values which should be passed from the backend to the frontend.
/// The grid must be passable between threads to allow the simulation to run in a separate thread
/// from the UI.
///
/// The implementation is not defined as a struct to allow flexible refactoring of the values'
/// container if such need is encountered (for performance reasons for example).
///
/// Note that this trait is meant to be a read-only protocol, aimed to be sent from backend to
/// frontend.
pub trait SimulationGrid<'a, T>: Send + Sync {
    /// Returns the dimensions of the grid (its length in the horizontal and vertical dimensions).
    ///
    /// # Return value
    /// A tuple whose first element is the grid's *horizontal* length, and whose second is the
    /// grid's *vertical* length.
    fn get_grid_dimensions(&self) -> (usize, usize);

    /// Retrieves a reference to a value at a specific location in the grid.
    ///
    /// # Arguments
    /// * `x` - The horizontal coordinate of the value in the grid.
    /// * `y` - The vertical coordinate of the value in the grid.
    ///
    /// # Return value
    /// An immutable reference to the grid value at coordinates (`x`, `y`).
    fn get_value_at(&self, x: usize, y: usize) -> &'a T;
}

/// Represents the different types of simulation data which can be sent from the backend to the
/// frontend.
pub enum SimulationData<'a> {
    /// A grid of pressure values.
    Pressure(Box<dyn SimulationGrid<'a, f64> + 'a>),
}
