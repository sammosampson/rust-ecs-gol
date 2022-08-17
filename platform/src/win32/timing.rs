
use winapi:: {
    um:: {
        winnt::*,
        profileapi::*,
    }
};

use super::App;

pub fn get_wall_clock() -> LARGE_INTEGER {    
    let mut result = LARGE_INTEGER::default();
    unsafe {
        QueryPerformanceCounter(&mut result);
    }
    result
}

impl App {
    pub fn get_seconds_elapsed(&self, start: LARGE_INTEGER, end: LARGE_INTEGER) -> f32 {
        unsafe {
            return (end.QuadPart() - start.QuadPart()) as f32 / self.perf_count_frequency as f32;
        }
    }
}