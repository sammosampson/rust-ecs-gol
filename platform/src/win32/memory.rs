use gol_engine::*;
use winapi::um::{memoryapi::VirtualAlloc, winnt::{MEM_RESERVE, MEM_COMMIT, PAGE_READWRITE}};

pub const HEAP_SIZE: usize = megabytes!(64);

pub struct VirtualMemoryChunkFactory;

impl MemoryChunkFactory for VirtualMemoryChunkFactory {
    fn create(&self) -> MemorySlab {     
        let base_address = terrabytes!(2) as MutableMemoryPointer;
        let total_size = HEAP_SIZE;
                
        let base_address = unsafe {
            VirtualAlloc(
            base_address, 
            total_size, 
            MEM_RESERVE|MEM_COMMIT, 
            PAGE_READWRITE
            )
        };

        MemorySlab { base_address, total_size }
    }
}

pub type FixedSizeBlockAllocator = SwitchableStrategyAllocator::
    <FixedSizeBlockAllocatorStrategy<BumpAllocatorStrategy>, VirtualMemoryChunkFactory>;

pub const fn fixed_size_block_allocator() -> FixedSizeBlockAllocator {
    SwitchableStrategyAllocator::new(
        FixedSizeBlockAllocatorStrategy::new(
            BumpAllocatorStrategy::new()), 
            VirtualMemoryChunkFactory)
}

pub fn get_total_allocated_memory_size() -> MemoryIndex {
    (&ALLOCATOR as &FixedSizeBlockAllocator).heap_size()
}

pub fn get_allocated_memory_block() -> MutableMemoryPointer {
    (&ALLOCATOR as &FixedSizeBlockAllocator).heap_start()
}


#[global_allocator]
pub static ALLOCATOR: FixedSizeBlockAllocator = fixed_size_block_allocator();