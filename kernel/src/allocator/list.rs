/*
 * A heap allocator that uses a linked list to manage free memory blocks.
 * It allows for dynamic memory allocation and deallocation.
 *
 * Author: Philipp Oppermann, https://os.phil-opp.com/allocator-designs/
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-13
 */

use alloc::alloc::{GlobalAlloc, Layout};
use log::info;
use crate::allocator::global::{align_up, Locked};
use core::mem::{size_of, align_of};
use core::ptr::null_mut;
use core::fmt::Write;

/// Header of a free block in the list allocator.
struct ListNode {
    /// Size of the memory block
    size: usize,

    /// &'static mut type semantically describes an owned object behind a pointer.
    /// Basically, it’s a Box without a destructor that frees the object at the end of the scope.
    /// Its lifetime is static, meaning it will live for the entire duration of the program.
    /// Of course, this is not true in reality, as we might delete the list node at some point.
    /// But the compiler does not know this.
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    /// Create a new ListNode with the given size and no next node.
    const fn new(size: usize) -> Self {
        ListNode { size, next: None }
    }

    /// Get the start address of the memory block.
    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    /// Get the end address of the memory block.
    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

/// A linked list allocator that uses a free list to manage memory.
pub struct LinkedListAllocator {
    head: ListNode,
    heap_start: usize,
    heap_end: usize,
}

impl LinkedListAllocator {
    /// Create a new empty linked list allocator.
    pub const fn new() -> LinkedListAllocator {
        LinkedListAllocator {
            head: ListNode::new(0),
            heap_start: 0,
            heap_end: 0,
        }
    }

    /// Initialize the allocator with the heap bounds given in the constructor.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        unsafe {
            self.add_free_block(heap_start, heap_size);
        }
    }

    /// Adds the given free memory block 'addr' to the free list.
    unsafe fn add_free_block(&mut self, addr: usize, size: usize) {
        let mut current = &mut self.head;

        // find position to insert
        while let Some(next) = current.next.as_ref() {
            if next.start_addr() >= addr {
                break;
            }

            current = current.next.as_mut().unwrap();
        }

        let end_addr = addr
            .checked_add(size)
            .expect("free block end address overflowed");

        // check which adjacent blocks can be merged
        let prev_adjacent = current.end_addr() == addr;
        let next_adjacent = current
            .next
            .as_ref()
            .map_or(false, |next| end_addr == next.start_addr());

        // merge prev
        if prev_adjacent {
            current.size += size;

            // also merge next
            if next_adjacent {
                let next_node = current.next.take().unwrap();
                current.size += next_node.size;
                current.next = next_node.next.take();
            }

            return;
        }

        let new_node = addr as *mut ListNode;

        unsafe {
            *new_node = ListNode {
                size,
                next: current.next.take(),
            };
        }

        current.next = Some(unsafe { &mut *new_node });

        // merge next
        if next_adjacent {
            let inserted = current.next.as_mut().unwrap();
            let next_node = inserted.next.take().unwrap();
            inserted.size += next_node.size;
            inserted.next = next_node.next.take();
        }
    }

    /// Search a free block with the given size and alignment and remove it from the list.
    fn find_free_block(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
        let mut current = &mut self.head;

        while let Some(_) = current.next.as_mut() {
            if let Ok(alloc_start) = Self::check_block_for_alloc(current.next.as_ref().unwrap(), size, align) {
                // current --> node == current.next
                let node = current.next.take().unwrap();
                // current --> node.next == current.next.next
                current.next = node.next.take();
                return Some((node, alloc_start));
            }
            current = current.next.as_mut().unwrap();
        }
        None
    }

    /// Check if the given block is large enough for an allocation with `size` and `align`.
    fn check_block_for_alloc(block: &ListNode, size: usize, align: usize) -> Result<usize,()> {
        let alloc_start = align_up(block.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        if alloc_end > block.end_addr() {
            return Err(());
        }
        Ok(alloc_start)
    }

    /// Adjust the given layout so that the resulting allocated memory
    /// block is also capable of storing a `ListNode`.
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(size_of::<ListNode>());

        (size, layout.align())
    }

    /// Dump the free list for debugging purposes.
    pub fn dump_free_list(&mut self) {
        println!("Free blocks:");
        let mut current = self.head.next.as_ref();
        while let Some(node) = current {
            println!("  block: start={:#x}, size={:#x}, end={:#x}", node.start_addr(), node.size, node.end_addr());
            current = node.next.as_ref();
        }
    }

    /// Allocate memory of the given size and alignment.
    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let (size, align) = Self::size_align(layout);

        if let Some((block, alloc_start)) = self.find_free_block(size, align) {
            let alloc_end = alloc_start + size;
            let excess_size = block.end_addr() - alloc_end;

            // If there's leftover space large enough to store a ListNode, add it back to the list
            if excess_size >= size_of::<ListNode>() {
                unsafe {
                    self.add_free_block(alloc_end, excess_size);
                }
            } else if excess_size > 0 {
                info!("Warning: leftover block of size {} got lost in limbo!", excess_size);
            }

            alloc_start as *mut u8
        } else {
            null_mut()
        }
    }

    /// Free the memory block at the given pointer with the given layout.
    pub unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let (size, _) = LinkedListAllocator::size_align(layout);

        unsafe {
            self.add_free_block(ptr as usize, size)
        }
    }
}

// Trait required by the Rust runtime for heap allocations
unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            self.lock().alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            self.lock().dealloc(ptr, layout);
        }
    }
}
