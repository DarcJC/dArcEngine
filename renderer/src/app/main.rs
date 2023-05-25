pub mod context;
pub mod stat;

use std::sync::RwLock;

use bevy_ecs::{world::World};
use context::ApplicationContext;
use darc_renderer::component::{GSCHEDULES, GWORLD};
use lazy_static::lazy_static;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

lazy_static! {
    static ref APPLICATION_CONTEXT: RwLock<ApplicationContext<'static>> = RwLock::new(ApplicationContext::new());
}

#[async_std::main]
async fn main() {
    start().await;
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
async fn start() {
    initialize_world().await;
    let mut schedule = GSCHEDULES.write().unwrap();
    schedule.add_system(main_loop_system);
    schedule.add_system(stat::fps_stat_system);
    drop(schedule);

    ApplicationContext::run(APPLICATION_CONTEXT.write().unwrap()).await;
}

async fn initialize_world() {
    let mut world = GWORLD.write().unwrap();
    world.init_resource::<stat::PerformanceStat>();

    initialize_logger();
    ApplicationContext::initialize(APPLICATION_CONTEXT.write().unwrap()).await;
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
