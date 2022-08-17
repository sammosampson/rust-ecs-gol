use std::ffi::*;
use crate::threads::*;


#[repr(C)]
pub struct DebugReadFileResult {
    pub contents_size: u32,
    pub contents: *mut c_void
}

pub type DebugPlatformReadEntireFile = fn(&mut ThreadContext, &str) -> Option<DebugReadFileResult>;
pub type DebugPlatformFreeFileMemory = fn(&mut ThreadContext, *mut c_void);
pub type DebugPlatformWriteEntireFile = fn(&mut ThreadContext, &str, u32, *mut c_void) -> bool;