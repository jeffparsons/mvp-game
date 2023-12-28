cargo_component_bindings::generate!();

use bindings::exports::jeffparsons::mvp_game::startup_mod_api;

struct Component;

impl startup_mod_api::Guest for Component {
    fn run() -> String {
        "Hello from Wasm Component!".to_string()
    }
}
