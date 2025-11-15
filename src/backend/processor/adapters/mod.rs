//! Module containing implementation of different adapters that can be used in the simulation.

use serde::Deserialize;

/// An adapter-specific configuration, through which the [`AdapterFactory`] enum will be able to construct
/// the needed adapter while abstracting which one it provided.
#[derive(Debug, Clone, Copy, Deserialize)]
pub enum AdapterConfiguration {}

/// A factory enum whose variants are wrappers for actual implementations of the [`SimulationDataAdapter`]
/// trait.
///
/// The factory will use the [`AdapterConfiguration`] given to it to create the matching adapter variant.
pub enum AdapterFactory {}

impl TryFrom<AdapterConfiguration> for AdapterFactory {
    type Error = anyhow::Error;

    fn try_from(value: AdapterConfiguration) -> Result<Self, Self::Error> {
        todo!("Implement actual adapters and create them here!")
    }
}
