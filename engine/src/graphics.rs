use std::ffi::*;

#[repr(C)]
pub struct GameOffscreenBuffer {
    pub memory: *mut c_void,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bytes_per_pixel: u32
}