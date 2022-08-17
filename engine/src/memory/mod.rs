mod locked;
mod allocator;
mod bump;
mod dynamic;

use std::ffi::*;

pub use dynamic::*;
pub use allocator::*;
pub use bump::*;

use crate::{files::*, threads::*};

#[macro_export]
macro_rules! kilobytes { ($kb:expr) => { $kb * 1024usize }; }
#[macro_export]
macro_rules! megabytes { ($mb:expr) => { $mb * kilobytes!(1024usize) }; }
#[macro_export]
macro_rules! gigabytes { ($gb:expr) => { $gb * megabytes!(1024usize) }; }
#[macro_export]
macro_rules! terrabytes { ($tb:expr) => { $tb * gigabytes!(1024usize) }; }

pub type MemoryIndex = usize;
pub type MutableMemoryPointer = *mut c_void;

#[repr(C)]
pub struct GameMemory {
    pub is_initialized: bool,
    pub root: MutableMemoryPointer,
    pub debug_platform_read_entire_file_func: Box<DebugPlatformReadEntireFile>, 
    pub debug_platform_free_file_memory_func: Box<DebugPlatformFreeFileMemory>, 
    pub debug_platform_write_entire_file_func: Box<DebugPlatformWriteEntireFile>, 
}

pub fn initialised(game_memory: &GameMemory) -> bool {
    game_memory.is_initialized
}

pub fn mark_as_initialised(game_memory: &mut GameMemory) {
    game_memory.is_initialized = true;
}

pub fn set_game_memory_root<T>(game_memory: &mut GameMemory, root: &mut T) {
    game_memory.root = (root as *mut T) as *mut c_void;
}
    
pub fn get_game_memory_root<T>(game_memory: &mut GameMemory) -> &mut T {
    let root = game_memory.root as *mut T;
    unsafe { &mut (*root) }   
}

impl GameMemory {
    pub fn debug_platform_read_entire_file(&self, thread_context: &mut ThreadContext, file_name: &str) -> Option<DebugReadFileResult> {
        (self.debug_platform_read_entire_file_func)(thread_context, file_name)
    }
     
    pub fn debug_platform_free_file_memory(&self, thread_context: &mut ThreadContext, memory: *mut c_void) {
        (self.debug_platform_free_file_memory_func)(thread_context, memory)
    }
     
    pub fn debug_platform_write_entire_file(&self, thread_context: &mut ThreadContext, file_name: &str, memory_size: u32, memory: *mut c_void) -> bool {
        (self.debug_platform_write_entire_file_func)(thread_context, file_name, memory_size, memory)
    }
}

pub trait MemoryChunkFactory {
    fn create(&self) -> MemorySlab;
}

pub struct MemorySlab {
    pub base_address: MutableMemoryPointer,
    pub total_size: MemoryIndex
}