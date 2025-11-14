//! Module holding the main data structure used in the simulation - a two-dimensional grid.

/// A two-dimensional grid holding some kind of data. Should be used to transport data around
/// the simulation.
pub struct Grid<T> {
    /// Width of the grid (horizontal length).
    width: usize,
    /// Height of the grid (vertical length).
    height: usize,
    /// The actual place where values are stored (this implementation uses a one-dimensional Vec).
    flat_grid: Vec<T>,
}

impl<D: Default> Grid<D> {
    /// Creates a new grid for any type which implements the `Default` trait.
    ///
    /// # Arguments:
    /// * `width` - The horizontal length of the grid.
    /// * `height` - The vertical length of the grid.
    ///
    /// # Type Arguments:
    /// * `D` - Any type which implements `Default`.
    ///
    /// # Return Value:
    /// A `Grid` whose values are initialized using the `Default` implementation of the type
    /// parameter `D`.
    pub fn new(width: usize, height: usize) -> Grid<D> {
        let flat_grid: Vec<D> = (0..width * height)
            .into_iter()
            .map(|_| Default::default())
            .collect();
        Grid {
            width,
            height,
            flat_grid,
        }
    }
}

impl <T> Grid<T> {
    /// Maps the natural two-dimensional coordinates in a grid to a one-dimensional index which
    /// refers to the equivalent position in the flat grid.
    fn calculate_flat_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// The horizontal length of the grid.
    pub fn width(&self) -> usize {
        self.width
    }

    /// The vertical length of the grid.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Retrieves a reference to the data located at `Grid(x, y)`.
    ///
    /// # Arguments:
    /// * `x` - Horizontal index in the grid.
    /// * `y` - Vertical index in the grid.
    ///
    /// # Return Value:
    /// A reference to the data at coordinates `x`, `y` in the grid, or `None` if such coordinates
    /// point to outside the grid.
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.flat_grid.get(self.calculate_flat_index(x, y))
    }

    /// Sets the data located at `Grid(x, y)`.
    ///
    /// # Arguments:
    /// * `x` - Horizontal index in the grid.
    /// * `y` - Vertical index in the grid.
    /// * `new_value` - The new value which will be stored at the given position in the grid.
    ///
    /// # Return Value:
    /// `Some(())` if the new value was stored at the given coordinates, or `None` if such
    /// coordinates point to outside the grid.
    pub fn set(&mut self, x: usize, y: usize, new_value: T) -> Option<()> {
        let flat_index = self.calculate_flat_index(x, y);
        self.flat_grid
            .get_mut(flat_index)
            .map(|grid_value| *grid_value = new_value)
    }
}

impl<T> From<Vec<Vec<T>>> for Grid<T> {
    fn from(raw_grid: Vec<Vec<T>>) -> Self {
        let height = raw_grid.len();
        if height == 0 {
            return Grid {
                width: 0,
                height: 0,
                flat_grid: Vec::new(),
            };
        }

        let width = raw_grid[0].len();
        if width == 0 {
            return Grid {
                width: 0,
                height: 0,
                flat_grid: Vec::new(),
            };
        }

        Grid {
            width,
            height,
            flat_grid: raw_grid.into_iter().flatten().collect(),
        }
    }
}