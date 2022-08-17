use std::ffi::*;

#[repr(C)]
pub struct GameSoundOutputBuffer {
    pub samples_per_second: u32,
    pub sample_count: u32,
    pub samples: *mut c_void
}