//! Module holding the main data structure used in the simulation - a two-dimensional grid.

/// A two-dimensional grid holding some kind of data. Should be used to transport data around
/// the simulation.
#[derive(Debug)]
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

impl<T> Grid<T> {
    /// Maps the natural two-dimensional coordinates in a grid to a one-dimensional index which
    /// refers to the equivalent position in the flat grid.
    fn calculate_flat_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// Turns the flat (true) index in a grid into an (x, y) index pair.
    const fn unflatten_index(flat_index: usize, grid_width: usize) -> (usize, usize) {
        (flat_index % grid_width, flat_index / grid_width)
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

    /// Retrieves a mutable reference to the data located at `Grid(x, y)`.
    ///
    /// # Arguments:
    /// * `x` - Horizontal index in the grid.
    /// * `y` - Vertical index in the grid.
    ///
    /// # Return Value:
    /// A mutable reference to the data at coordinates `x`, `y` in the grid, or `None` if such
    /// coordinates point to outside the grid.
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        let flat_index = self.calculate_flat_index(x, y);
        self.flat_grid.get_mut(flat_index)
    }

    /// Retrieves a reference to the data located at `Grid(x, y)`, without validating
    /// the coordinates are valid.
    /// 
    /// This function is more performant than [Grid::get], but will cause undefined behavior if
    /// the coordinates are out of bounds.
    /// 
    /// # Arguments:
    /// * `x` - Horizontal index in the grid.
    /// * `y` - Vertical index in the grid.
    ///
    /// # Return Value:
    /// A reference to the data at coordinates `x`, `y` in the grid.
    pub unsafe fn get_unchecked(&self, x: usize, y: usize) -> &T {
        unsafe {
            self.flat_grid.get_unchecked(self.calculate_flat_index(x, y))
        }
    }

    /// Retrieves a mutable reference to the data located at `Grid(x, y)`, without validating
    /// the coordinates are valid.
    /// 
    /// This function is more performant than [Grid::get_mut], but will cause undefined behavior if
    /// the coordinates are out of bounds.
    /// 
    /// # Arguments:
    /// * `x` - Horizontal index in the grid.
    /// * `y` - Vertical index in the grid.
    ///
    /// # Return Value:
    /// A mutable reference to the data at coordinates `x`, `y` in the grid.
    pub unsafe fn get_unchecked_mut(&mut self, x: usize, y: usize) -> &mut T {
        let flat_index = self.calculate_flat_index(x, y);
        unsafe {
            self.flat_grid.get_unchecked_mut(flat_index)
        }
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

    /// Applies a function for each cell in the grid, immutably.
    /// 
    /// # Arguments:
    /// * `consumer` - A consuming function that accepts a reference to some cell in the grid, and the
    ///                cell's coordinates (x, y), and performs some computations with them.
    pub fn for_each<F>(&self, consumer: F)
    where 
        F: Fn(&T, (usize, usize)) -> ()
    {
        self.flat_grid
            .iter()
            .enumerate()
            .for_each(|(flat_index, value)| {
                consumer(value, Self::unflatten_index(flat_index, self.width))
            });
    }

    /// Applies a function for each cell in the grid, mutably.
    /// 
    /// # Arguments:
    /// * `consumer` - A consuming function that accepts a mutable reference to some cell in the grid,
    ///                and the cell's coordinates (x, y), and performs some computations with them.
    ///                The passed function may change the cell.
    pub fn for_each_mut<F>(&mut self, consumer: F)
    where 
        F: Fn(&mut T, (usize, usize))
    {
        self.flat_grid
            .iter_mut()
            .enumerate()
            .for_each(|(flat_index, value)| {
                consumer(value, Self::unflatten_index(flat_index, self.width))
            });
    }

    /// Changes the value in each cell by calling `filler` again and again.
    pub fn set_all_with<F: FnMut() -> T>(&mut self, filler: F) {
        self.flat_grid.fill_with(filler);
    }
}

impl <T> Grid<T>
where
    T: Copy
{
    /// Changes the value in each cell to `value`.
    pub fn set_all(&mut self, value: T) {
        self.flat_grid.fill(value);
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
