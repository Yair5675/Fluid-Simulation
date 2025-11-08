mod frontend;
mod backend;
pub mod ipc;

use yew::prelude::*;
use crate::frontend::configuration;

fn main() {
    let config: configuration::FrontendConfiguration = Default::default();
    yew::Renderer::<frontend::FrontendComponent>::with_props(config).render();
}
