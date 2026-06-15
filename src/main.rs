use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::p2p::node::RetroNode;
use crate::pages::landing::LandingPage;
use crate::pages::retro::RetroPage; // Node-Typ importieren

mod components;
mod models;
mod p2p;
mod pages;

pub(crate) const ALPN: &[u8] = b"iroh/p2p-retro/0";

fn main() {
    let config = tracing_wasm::WASMLayerConfigBuilder::new()
        .set_max_level(tracing::Level::WARN)
        .build();
    tracing_wasm::set_as_global_default_with_config(config);

    leptos::mount::mount_to_body(|| {
        let node_signal = RwSignal::new(Option::<RetroNode>::None);
        provide_context(node_signal);

        leptos::view! {
            <div style:min-height="100vh">
                <Router base="/iroh-retro">
                    <Routes fallback=|| "404. Not found.">
                        <Route path=path!("/") view=LandingPage />
                        <Route path=path!("/session/:id") view=RetroPage />
                    </Routes>
                </Router>
            </div>
        }
    })
}
