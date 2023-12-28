use anyhow::Context;
use bevy::{
    app::{App, Startup},
    ecs::system::{ResMut, Resource},
    DefaultPlugins, MinimalPlugins,
};
use wasmtime_wasi::preview2::{Table, WasiCtx, WasiCtxBuilder, WasiView};

mod startup_mod {
    wasmtime::component::bindgen!("startup-mod");
}

struct StartupModState {
    wasi_ctx: WasiCtx,
    table: Table,
}

impl WasiView for StartupModState {
    fn table(&self) -> &Table {
        &self.table
    }

    fn table_mut(&mut self) -> &mut Table {
        &mut self.table
    }

    fn ctx(&self) -> &WasiCtx {
        &self.wasi_ctx
    }

    fn ctx_mut(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
}

fn main() -> anyhow::Result<()> {
    let mut config = wasmtime::Config::new();
    config.wasm_component_model(true);
    let engine = wasmtime::Engine::new(&config)?;
    let component = wasmtime::component::Component::from_file(
        &engine,
        "wasm-components/target/wasm32-wasi/debug/test_startup_mod.wasm",
    )?;
    let mut linker = wasmtime::component::Linker::new(&engine);

    wasmtime_wasi::preview2::command::sync::add_to_linker(&mut linker)
        .context("Failed to set up WASI in component linker")?;

    let mut store = wasmtime::Store::new(
        &engine,
        StartupModState {
            wasi_ctx: WasiCtxBuilder::new().build(),
            table: Table::new(),
        },
    );
    let (bindings, _) = startup_mod::StartupMod::instantiate(&mut store, &component, &linker)?;

    let mut app = App::new();

    app.insert_resource(StartupMods {
        startup_mods: vec![StartupMod { bindings, store }],
    });

    // TODO: Not sure if I want to support an env var for this.
    // Maybe have a CLI flag but also support this for convenience
    // when developing.
    if std::env::var("MVP_HEADLESS").is_ok() {
        // TODO: Not necessarily `MinimalPlugins` -- just not GUI.
        // Need to inspect the difference in what each includes.
        app.add_plugins(MinimalPlugins);
    } else {
        app.add_plugins(DefaultPlugins);
    }

    app.add_systems(Startup, startup_mods_system);

    app.run();

    Ok(())
}

fn startup_mods_system(mut mods: ResMut<StartupMods>) {
    for startup_mod in &mut mods.startup_mods {
        let Ok(message) = startup_mod
            .bindings
            .jeffparsons_mvp_game_startup_mod_api()
            .call_run(&mut startup_mod.store)
        else {
            eprintln!("ERROR calling startup mod");
            continue;
        };
        println!("Message from startup mod: {message}");
    }
}

// TODO: What to call these? Not "mods". And I don't like "handlers".
#[derive(Resource)]
struct StartupMods {
    startup_mods: Vec<StartupMod>,
}

struct StartupMod {
    bindings: startup_mod::StartupMod,
    store: wasmtime::Store<StartupModState>,
}
