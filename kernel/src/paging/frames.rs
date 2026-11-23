/*
 * Contains the interface definition for an Interrupt Service Routine (ISR).
 * This must be implemented by a device driver if it needs to handle interrupts.
 * The ISR is registered using the `register()` function in `intdispatcher.rs`.
 *
 * Author: Michael Schoetter, Heinrich Heine University Duesseldorf, 2024-01-21
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2025-10-31
 * License: GPLv3
 */

use core::fmt;
use core::ops::{Add, Sub};
use crate::allocator::global::align_up;
use crate::consts::PAGE_SIZE;
use crate::library::spinlock::Spinlock as Mutex;

pub static FRAME_ALLOCATOR: Mutex<PfListAllocator> = Mutex::new(PfListAllocator::new());

/// Represents a physical address in memory and allows accessing it via pointers.
/// Basic arithmetic operations are implemented for easy address manipulation.
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct PhysAddr(u64);

impl PhysAddr {
    pub fn new(addr: u64) -> Self {
        PhysAddr(addr)
    }

    pub fn raw(&self) -> u64 {
        self.0
    }

    pub fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    pub fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }
}

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Phys(0x{:016x})", self.0)
    }
}

impl From<PhysAddr> for u64 {
    fn from(addr: PhysAddr) -> Self {
        addr.0
    }
}

impl Add<PhysAddr> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: PhysAddr) -> Self::Output {
        let res = self.0.checked_add(rhs.0).unwrap();
        PhysAddr(res)
    }
}

impl Sub<PhysAddr> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: PhysAddr) -> Self::Output {
        let res = self.0.checked_sub(rhs.0).unwrap();
        PhysAddr(res)
    }
}

impl Add<usize> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: usize) -> Self::Output {
        let res = self.0.checked_add(rhs as u64).unwrap();
        PhysAddr(res)
    }
}

impl Sub<usize> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: usize) -> Self::Output {
        let res = self.0.checked_sub(rhs as u64).unwrap();
        PhysAddr(res)
    }
}

impl Add<u64> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: u64) -> Self::Output {
        let res = self.0.checked_add(rhs).unwrap();
        PhysAddr(res)
    }
}

impl Sub<u64> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: u64) -> Self::Output {
        let res = self.0.checked_sub(rhs).unwrap();
        PhysAddr(res)
    }
}

/// A node in the physical frame free list.
/// Contains the size of the free block and a pointer to the next node.
struct PfListNode {
    size: usize,
    next: Option<&'static mut PfListNode>
}

impl PfListNode {
    const fn new(size: usize) -> Self {
        PfListNode { size, next: None }
    }

    fn start_addr(&self) -> PhysAddr {
        PhysAddr::new(self as *const Self as u64)
    }

    fn end_addr(&self) -> PhysAddr {
        self.start_addr() + self.size
    }
}

/// A physical frame allocator that uses a linked list to manage free memory blocks.
/// Memory blocks are always aligned to PAGE_SIZE (4096 bytes).
pub struct PfListAllocator {
    head: PfListNode
}

impl PfListAllocator {
    /// Create a new empty physical frame list allocator.
    pub const fn new() -> PfListAllocator {
        PfListAllocator {
            head: PfListNode::new(0)
        }
    }

    /// Adds the given free memory block 'addr' to the free list.
    unsafe fn add_free_block(&mut self, addr: PhysAddr, size: usize) {
        let mut current = &mut self.head;

        // find position to insert
        while let Some(next) = current.next.as_ref() {
            if next.start_addr() >= addr {
                break;
            }

            current = current.next.as_mut().unwrap();
        }

        let end_addr = addr + size;

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

        // the metadata is stored directly in the first frame of the free block
        let new_node = addr.as_mut_ptr::<PfListNode>();

        unsafe {
            *new_node = PfListNode {
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
    fn find_free_block(&mut self, size: usize, align: usize) -> Option<(&'static mut PfListNode, PhysAddr)> {
        let mut current = &mut self.head;

        while current.next.as_ref().is_some() {
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
    fn check_block_for_alloc(block: &PfListNode, size: usize, align: usize) -> Result<PhysAddr, ()> {
        let alloc_start = PhysAddr::new(align_up(block.start_addr().raw() as usize, align) as u64);
        let alloc_end = alloc_start.raw().checked_add(size as u64).ok_or(())?;

        if alloc_end > block.end_addr().raw() {
            return Err(());
        }
        Ok(alloc_start)
    }

    /// Convert a frame count into a size that is a multiple of PAGE_SIZE and
    /// return the required alignment along with it.
    fn size_align(num_frames: usize) -> Option<(usize, usize)> {
        let size = num_frames.checked_mul(PAGE_SIZE)?;
        Some((size, PAGE_SIZE))
    }

    /// Try to allocate a block of 'num_frames' physical frames.
    /// Returns the starting physical address of the allocated block on success.
    /// The found block is filled with zeroes. If the block is larger than requested,
    /// the remaining part is added back to the free list.
    /// If no suitable block is found, returns None.
    pub unsafe fn alloc_block(&mut self, num_frames: usize) -> Option<PhysAddr> {
        if num_frames == 0 {
            return None;
        }
        let (size, align) = Self::size_align(num_frames)?;

        if let Some((block, alloc_start)) = self.find_free_block(size, align) {
            let alloc_end = alloc_start + size;
            let excess_size = (block.end_addr().raw() - alloc_end.raw()) as usize;

            // Frames always have the same size, so there can be no unusable remainder.
            if excess_size >= PAGE_SIZE {
                unsafe {
                    self.add_free_block(alloc_end, excess_size);
                }
            } else {
                assert_eq!(excess_size, 0, "leftover memory is not a full frame");
            }

            // check for double alloc
            let mut current = self.head.next.as_ref();
            while let Some(node) = current {
                assert!(
                    node.start_addr().raw() >= alloc_end.raw() || node.end_addr().raw() <= alloc_start.raw(),
                    "allocated block is still in the free list"
                );
                current = node.next.as_ref();
            }

            // This helps to detect illegal pointer accesses, as null pointer accesses can be detected via paging.
            unsafe {
                core::ptr::write_bytes(alloc_start.as_mut_ptr::<u8>(), 0, size);
            }

            Some(alloc_start)
        } else {
            None
        }
    }

    /// Free a previously allocated block of 'num_frames' physical frames starting at 'addr'.
    /// The address must be aligned to PAGE_SIZE (4096 bytes).
    /// The freed block is merged with adjacent free blocks if possible.
    pub unsafe fn free_block(&mut self, addr: PhysAddr, num_frames: usize) {
        assert_eq!(addr.raw() % PAGE_SIZE as u64, 0, "physical address must be page aligned");
        assert!(num_frames > 0, "cannot free zero frames");
        let (size, _) = Self::size_align(num_frames).expect("frame count overflowed");

        // check for double free
        let end = addr.raw() + size as u64;
        let mut current = self.head.next.as_ref();
        while let Some(node) = current {
            assert!(
                node.start_addr().raw() >= end || node.end_addr().raw() <= addr.raw(),
                "block is already free"
            );
            current = node.next.as_ref();
        }

        unsafe {
            self.add_free_block(addr, size);
        }
    }

    /// Print the list of free physical memory.
    pub fn dump_free_list(&self) {
        println!("Free physical memory blocks:");
        let mut current = self.head.next.as_ref();
        while let Some(node) = current {
            println!("  block: start={:#x}, size={} frames, end={:#x}", node.start_addr().raw(), node.size / PAGE_SIZE, node.end_addr().raw());
            current = node.next.as_ref();
        }
    }
}
