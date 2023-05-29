use std::time::{SystemTime, Duration};
use bevy::prelude::{Resource, ResMut, Plugin};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;


#[derive(Resource, Debug, Clone, Copy, PartialEq)]
struct PerformanceStat {
    pub current_frame: u64,
    pub last_frame_time: f64,
}

impl Default for PerformanceStat {
    fn default() -> Self {
        Self {
            current_frame: 0,
            last_frame_time: performance_now(),
        }
    }
}

impl PerformanceStat {
    pub fn tick(&mut self) -> Result<std::time::Duration, std::time::SystemTimeError> {
        self.current_frame += 1;
        let last_frame_time = self.last_frame_time;
        self.last_frame_time = performance_now();
        Ok(Duration::from_micros( (self.last_frame_time - last_frame_time) as u64 ))
    }
}

fn fps_stat_system(mut stat: ResMut<PerformanceStat>) {
    if let Ok(duration) = stat.tick() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            print!("\x1b[2J");
            print!("\x1b[H");
            println!("frame time: {:?}", duration);
        }

        #[cfg(target_arch = "wasm32")]
        {
            log::log!(log::Level::Debug, "frame time: {:?}", duration);
        }
    }
}


/// WASM only has millisecond timestamp.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(inline_js = r#"
export function performance_now() {
  return performance.now() * 1000.0;
}"#)]
extern "C" {
    fn performance_now() -> f64;
}

/// Get current timestamp(monotonic) in microsecnod.
#[cfg(not(target_arch = "wasm32"))]
fn performance_now() -> f64 {
    use std::time::UNIX_EPOCH;

    SystemTime::now().duration_since(UNIX_EPOCH).expect("Could not get system time.").as_micros() as f64
}

pub struct PerformenceStatPlugin;

impl Plugin for PerformenceStatPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
        .init_resource::<PerformanceStat>()
        .add_system(fps_stat_system)
        ;
    }
}
