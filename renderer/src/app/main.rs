pub mod stat;

use bevy::{prelude::*, DefaultPlugins, render::{RenderPlugin, settings::WgpuSettings}};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[async_std::main]
async fn main() {
    start().await;
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
async fn start() {
    initialize_world().await;
    App::default()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            wgpu_settings: WgpuSettings {
                backends: None,
                ..default()
            },
        }))
        .add_system(main_loop_system)
        // .add_plugin(PerformenceStatPlugin)
        .add_plugin(darc_renderer::render::RenderPlugin)
        .run();
}

async fn initialize_world() {
    initialize_logger();
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
