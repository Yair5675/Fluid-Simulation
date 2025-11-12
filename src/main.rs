mod backend;
mod frontend;
pub mod ipc;

use yew::prelude::*;

fn main() {
    let config: frontend::configuration::FrontendConfiguration = Default::default();
    yew::Renderer::<frontend::FrontendComponent>::with_props(config).render();
}
