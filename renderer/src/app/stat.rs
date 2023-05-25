use std::time::SystemTime;

use bevy_ecs::system::Resource;


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
