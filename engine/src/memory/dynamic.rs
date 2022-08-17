use std::alloc::Layout;
use std::mem::{align_of, size_of};

use super::*;
use super::allocator::*;

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536];

fn page_block_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}

pub struct FixedSizeBlockAllocatorStrategy<FA: AllocatorStrategy> {
    page_blocks: [Option<&'static mut MemoryPageNode>; BLOCK_SIZES.len()],
    fallback_allocator: FA
}


impl<FA: AllocatorStrategy> FixedSizeBlockAllocatorStrategy<FA> {
    pub const fn new(fallback_allocator: FA) -> Self {
        const EMPTY: Option<&'static mut MemoryPageNode> = None;
        Self {
            page_blocks: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator
        }
    }
}

impl<FA: AllocatorStrategy> AllocatorStrategy for FixedSizeBlockAllocatorStrategy<FA> {
    fn initialised(&self) -> bool {
        self.fallback_allocator.initialised()
    }

    fn allocated(&self) -> usize {
        self.fallback_allocator.allocated()
    }

    fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size)
    }

    fn heap_size(&self) -> MemoryIndex {
        self.fallback_allocator.heap_size()
    }

    fn heap_start(&self) -> MutableMemoryPointer {
        self.fallback_allocator.heap_start()
    }

    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        match page_block_index(&layout) {
            Some(index) => {
                match self.page_blocks[index].take() {
                    Some(page) => {
                        self.page_blocks[index] = page.next.take();
                        page as *mut MemoryPageNode as *mut u8
                    }
                    None => {
                        let block_size = BLOCK_SIZES[index];
                        let block_align = block_size;
                        let layout = Layout::from_size_align(block_size, block_align).unwrap();
                        self.fallback_allocator.alloc(layout)
                    }
                }
            }
            None => self.fallback_allocator.alloc(layout),
        }
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        match page_block_index(&layout) {
            Some(index) => {
                let new_node = MemoryPageNode {
                    next: self.page_blocks[index].take(),
                };
                // verify that block has size and alignment required for storing node
                assert!(size_of::<MemoryPageNode>() <= BLOCK_SIZES[index]);
                assert!(align_of::<MemoryPageNode>() <= BLOCK_SIZES[index]);
                let new_node_ptr = ptr as *mut MemoryPageNode;
                new_node_ptr.write(new_node);
                self.page_blocks[index] = Some(&mut *new_node_ptr);
            }
            None => {
                self.fallback_allocator.dealloc(ptr, layout);
            }
        }
    }
}

struct MemoryPageNode {
    next: Option<&'static mut MemoryPageNode>,
}
