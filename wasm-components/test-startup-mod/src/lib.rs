cargo_component_bindings::generate!();

use bindings::exports::jeffparsons::mvp_game::startup_mod_api;
use bindings::jeffparsons::mvp_game::mvp_api::Commands;

struct Component;

impl startup_mod_api::Guest for Component {
    fn run(commands: &Commands) -> String {
        commands.spawn_stuff();
        "Hello from Wasm Component!".to_string()
    }
}
