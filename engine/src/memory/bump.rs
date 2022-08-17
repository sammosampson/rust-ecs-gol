

use std::{
    alloc::Layout,
    ptr
};

use super::*;
use super::allocator::*;

pub struct BumpAllocatorStrategy {
    pub initialised: bool,
    allocated: usize,
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocatorStrategy {
    pub const fn new() -> Self {
        BumpAllocatorStrategy {
            initialised: false,
            allocated: 0,
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }
}

impl AllocatorStrategy for BumpAllocatorStrategy {
    fn initialised(&self) -> bool {
        self.initialised
    }

    fn allocated(&self) -> usize {
        self.allocated
    }

    fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
        self.initialised = true;
    }

    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let alloc_start = align_up(self.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > self.heap_end {
            ptr::null_mut() // out of memory
        } else {
            self.next = alloc_end;
            self.allocations += 1;
            self.allocated = alloc_end - alloc_start;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
        self.allocations -= 1;
        if self.allocations == 0 {
            self.next = self.heap_start;
        }
    }

    fn heap_size(&self) -> MemoryIndex {
        self.heap_end - self.heap_start
    }

    fn heap_start(&self) -> MutableMemoryPointer {
        self.heap_start as MutableMemoryPointer
    }
}