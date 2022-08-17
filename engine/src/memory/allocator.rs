use std::{
    alloc::{
        GlobalAlloc,
        Layout
    }
};

use super::*;
use super::locked::*;

pub struct SwitchableStrategyAllocator<A:AllocatorStrategy, M:MemoryChunkFactory> {
    allocator_strategy: Locked<A>,
    memory_chunk_factory: M
}

impl<A:AllocatorStrategy, M:MemoryChunkFactory> SwitchableStrategyAllocator<A, M> {
    pub const fn new(allocator_strategy: A, memory_chunk_factory: M) -> Self {
        Self { 
            allocator_strategy: Locked::new(allocator_strategy),
            memory_chunk_factory
        }
    }

    pub fn allocated(&self) -> MemoryIndex {
        self.allocator_strategy.lock().allocated()
    }
    
    pub fn heap_size(&self) -> MemoryIndex {
        self.allocator_strategy.lock().heap_size()
    }
    
    pub fn heap_start(&self) -> MutableMemoryPointer {
        self.allocator_strategy.lock().heap_start()
    }
}

unsafe impl<A:AllocatorStrategy, M:MemoryChunkFactory> GlobalAlloc for SwitchableStrategyAllocator<A, M> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.allocator_strategy.lock(); 
        if !allocator.initialised() {
            let slab = self.memory_chunk_factory.create(); 
            allocator.init(slab.base_address as usize, slab.total_size);
        }
        allocator.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.allocator_strategy.lock().dealloc(ptr, layout);
    }
}

pub trait AllocatorStrategy {
    fn initialised(&self) -> bool;
    fn allocated(&self) -> MemoryIndex;
    fn heap_size(&self) -> MemoryIndex;
    fn heap_start(&self) -> MutableMemoryPointer;
    fn init(&mut self, heap_start: usize, heap_size: usize);
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8;
    unsafe fn dealloc(&mut self, ptr: *mut u8, _layout: Layout);
}

pub fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}