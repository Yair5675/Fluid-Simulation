//! Module containing useful simulation data-types that may appear throughout the backend.

/// Represents a 2-D velocity vector.
/// * `x` is the horizontal velocity component.
/// * `y` is the vertical velocity component.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Velocity {
    x: f64,
    y: f64,
}

impl Default for Velocity {
    fn default() -> Self {
        Velocity { x: 0.0, y: 0.0 }
    }
}
