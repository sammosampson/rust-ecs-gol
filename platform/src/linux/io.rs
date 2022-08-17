  
use std::ffi::*;    
pub use super::*;
use crate::prelude::*;
     
#[cfg(feature="gol-internal")]
pub fn debug_platform_read_entire_file(file_name: &str) -> Option<DebugReadFileResult> {
    todo!();
}
     
#[cfg(feature="gol-internal")]
pub fn debug_platform_free_file_memory(memory: *mut c_void) {
    todo!();
}
     
#[cfg(feature="gol-internal")]
pub fn debug_platform_write_entire_file(file_name: &str, memory_size: u32, memory: *mut c_void) {
    todo!();
}