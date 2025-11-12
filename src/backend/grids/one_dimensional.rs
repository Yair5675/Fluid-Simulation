use crate::backend::grids::Grid;

/// An implementation of a two-dimensional grid using a one-dimensional `Vec`.
///
/// A generic is tossed in to make it, well, generic.
pub struct OneDimensionalGrid<T> {
    width: usize,
    height: usize,
    flat_grid: Vec<T>,
}

impl<D: Default> OneDimensionalGrid<D> {
    /// Creates a new one-dimensional grid for any type which implements the `Default` trait.
    ///
    /// # Arguments:
    /// * `width` - The horizontal length of the grid.
    /// * `height` - The vertical length of the grid.
    ///
    /// # Type Arguments:
    /// * `D` - Any type which implements `Default`.
    ///
    /// # Return Value:
    /// A `OneDimensionalGrid` whose values are initialized using the `Default` implementation of
    /// the type argument `D`
    pub fn new(width: usize, height: usize) -> OneDimensionalGrid<D> {
        let flat_grid: Vec<D> = (0..width * height)
            .into_iter()
            .map(|_| Default::default())
            .collect();
        OneDimensionalGrid {
            width,
            height,
            flat_grid,
        }
    }
}

impl<T> OneDimensionalGrid<T> {
    /// Maps the natural two-dimensional coordinates in a grid to a one-dimensional index which
    /// refers to the equivalent position in the flat grid.
    fn calculate_flat_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// The horizontal length of the grid.
    fn width(&self) -> usize {
        self.width
    }

    /// The vertical length of the grid.
    fn height(&self) -> usize {
        self.height
    }
}

impl<T> Grid for OneDimensionalGrid<T> {
    type GridValue = T;

    fn get(&self, x: usize, y: usize) -> Option<&Self::GridValue> {
        self.flat_grid.get(self.calculate_flat_index(x, y))
    }

    fn set(&mut self, x: usize, y: usize, new_value: Self::GridValue) -> Option<()> {
        let flat_index = self.calculate_flat_index(x, y);
        self.flat_grid
            .get_mut(flat_index)
            .map(|grid_value| *grid_value = new_value)
    }
}

impl<T> From<Vec<Vec<T>>> for OneDimensionalGrid<T> {
    fn from(raw_grid: Vec<Vec<T>>) -> Self {
        let height = raw_grid.len();
        if height == 0 {
            return OneDimensionalGrid {
                width: 0,
                height: 0,
                flat_grid: Vec::new(),
            };
        }

        let width = raw_grid[0].len();
        if width == 0 {
            return OneDimensionalGrid {
                width: 0,
                height: 0,
                flat_grid: Vec::new(),
            };
        }

        OneDimensionalGrid {
            width,
            height,
            flat_grid: raw_grid.into_iter().flatten().collect(),
        }
    }
}
