//! Handles the visual part of our fluid simulation.

pub mod configuration;

use yew::prelude::*;

/// Main component of the fluid simulation's frontend.
///
/// This functional component abstracts how different frontends are chosen through the
/// configuration.
#[function_component(FrontendComponent)]
pub fn frontend_component(config: &configuration::FrontendConfiguration) -> Html {
    let frontend = get_frontend_from_config(config);
    frontend.get_html()
}

/// A factory-like function which returns a `FluidSimulationFrontend` trait object, based on the
/// given configuration.
///
/// # Arguments
/// * `_config` - Configuration for the returned `FluidSimulationFrontend`.
///
/// # Return value
/// A frontend based on the given configuration.
fn get_frontend_from_config(_config: &configuration::FrontendConfiguration) -> Box<dyn FluidSimulationFrontend> {
    // TODO: Replace with an actual implementation of factory-like creation
    Box::new(PocFrontend {})
}


/// Trait that abstracts different frontends and allows the program to create completely different
/// GUIs.
///
/// The `Html` object returned from the `get_html` function is expected to contain the frontend's
/// component within it, so it can be rendered.
trait FluidSimulationFrontend {
    fn get_html(&self) -> Html;
}


struct PocFrontend {}

impl FluidSimulationFrontend for PocFrontend {
    fn get_html(&self) -> Html {
        html! {
            <div>
                <p>{"Hello, world!"}</p>
            </div>
        }
    }

}
