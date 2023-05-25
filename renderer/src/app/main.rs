pub mod context;


use std::sync::RwLock;

use async_std::task::block_on;
use bevy_ecs::{world::World};
use context::ApplicationContext;
use darc_renderer::component::{GWORLD, GSCHEDULES};
use lazy_static::lazy_static;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

lazy_static! {
    static ref APPLICATION_CONTEXT: RwLock<ApplicationContext> = RwLock::new(block_on(ApplicationContext::new()));
}

#[async_std::main]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
async fn main() {
    initialize_world().await;
    let mut schedule = GSCHEDULES.write().await;
    schedule.add_system(main_loop_system);
    drop(schedule);

    APPLICATION_CONTEXT.write().unwrap().run();
}

async fn initialize_world() {
    initialize_logger();
    APPLICATION_CONTEXT.write().unwrap().initialize().await;
}

fn initialize_logger() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Trace).unwrap();
        } else {
            env_logger::init();
        }
    }
}

fn main_loop_system(_world: &mut World) {
}
