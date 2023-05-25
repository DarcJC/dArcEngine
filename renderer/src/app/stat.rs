use std::time::SystemTime;

use bevy_ecs::system::{Resource, ResMut};


#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerformanceStat {
    pub current_frame: u64,
    pub last_frame_time: SystemTime,
}

impl Default for PerformanceStat {
    fn default() -> Self {
        Self {
            current_frame: 0,
            last_frame_time: SystemTime::now(),
        }
    }
}

impl PerformanceStat {
    pub fn tick(&mut self) -> Result<std::time::Duration, std::time::SystemTimeError> {
        self.current_frame += 1;
        self.last_frame_time = SystemTime::now();
        self.last_frame_time.elapsed()
    }
}

pub fn fps_stat_system(mut stat: ResMut<PerformanceStat>) {
    if let Ok(duration) = stat.tick() {
        print!("\x1b[2J");
        print!("\x1b[H");
        println!("frame time: {:?}", duration);
    }
}
