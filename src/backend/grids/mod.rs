//! A module defining grids in the fluid simulation - holding data in a 2-dimensional data
//! structure.

mod staggered;

// Expose grids directly to save users headache:
#[allow(unused_imports)]
pub use staggered::StaggeredVelocityField;
