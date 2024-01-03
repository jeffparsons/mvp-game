mod ref_mutex;

use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use bevy::{
    app::{App, Startup},
    ecs::system::{Commands, ResMut, Resource},
    DefaultPlugins, MinimalPlugins,
};
use ref_mutex::RefMutMutex;
use startup_mod::jeffparsons::mvp_game::mvp_api::{self, Host, HostCommands};
use wasmtime_wasi::preview2::{Table, WasiCtx, WasiCtxBuilder, WasiView};

mod startup_mod {
    wasmtime::component::bindgen!("startup-mod");
}

struct StartupModState {
    wasi_ctx: WasiCtx,
    table: Table,
    commands_table: HashMap<u32, ImplCommands>,
    commands_mutex: Arc<RefMutMutex>,
}

struct ImplCommands {}

impl Host for StartupModState {}

impl HostCommands for StartupModState {
    fn spawn_stuff(
        &mut self,
        _self_: wasmtime::component::Resource<mvp_api::Commands>,
    ) -> wasmtime::Result<()> {
        let mut commands = self.commands_mutex.lock();
        let commands = commands.as_mut().unwrap();

        println!("I'm going to spawn stuff...!");
        // println!("(Another print to force a rebuild)");

        // TODO: Actually spawn a bunch of stuff.
        commands.spawn_empty();

        Ok(())
    }

    fn drop(
        &mut self,
        rep: wasmtime::component::Resource<mvp_api::Commands>,
    ) -> wasmtime::Result<()> {
        self.commands_table.remove(&rep.rep());

        Ok(())
    }
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
    let mut linker: wasmtime::component::Linker<StartupModState> =
        wasmtime::component::Linker::new(&engine);

    wasmtime_wasi::preview2::command::sync::add_to_linker(&mut linker)
        .context("Failed to set up WASI in component linker")?;

    startup_mod::StartupMod::add_to_linker(&mut linker, |state| state)
        .context("Failed to set up startup mod world in component linker")?;

    let commands_mutex = Arc::new(RefMutMutex::new());

    let mut store = wasmtime::Store::new(
        &engine,
        StartupModState {
            wasi_ctx: WasiCtxBuilder::new().build(),
            table: Table::new(),
            commands_table: HashMap::new(),
            commands_mutex: commands_mutex.clone(),
        },
    );
    let (bindings, _) = startup_mod::StartupMod::instantiate(&mut store, &component, &linker)?;

    let mut app = App::new();

    app.insert_resource(StartupMods {
        startup_mods: vec![StartupMod { bindings, store }],
        commands_mutex,
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

fn startup_mods_system(mut commands: Commands, mut mods: ResMut<StartupMods>) {
    let mods = &mut *mods;

    mods.commands_mutex.share(&mut commands, || {
        for startup_mod in &mut mods.startup_mods {
            // TODO: Don't create a new one each time.
            // In fact, we can probably just reuse _one_ for all calls,
            // and so we don't even really need to store it in a "table".
            let commands_table = &mut startup_mod.store.data_mut().commands_table;
            let handle = wasmtime::component::Resource::<mvp_api::Commands>::new_own(
                commands_table.len() as u32,
            );
            commands_table.insert(handle.rep(), ImplCommands {});

            let Ok(message) = startup_mod
                .bindings
                .jeffparsons_mvp_game_startup_mod_api()
                .call_run(&mut startup_mod.store, handle)
            else {
                eprintln!("ERROR calling startup mod");
                continue;
            };
            println!("Message from startup mod: {message}");
        }
    });
}

// TODO: What to call these? Not "mods". And I don't like "handlers".
#[derive(Resource)]
struct StartupMods {
    startup_mods: Vec<StartupMod>,
    commands_mutex: Arc<RefMutMutex>,
}

struct StartupMod {
    bindings: startup_mod::StartupMod,
    store: wasmtime::Store<StartupModState>,
}
