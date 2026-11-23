/*
 * Contains a demo for physical memory allocations.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-03-06
 * License: GPLv3
 */

use alloc::vec::Vec;
use crate::consts::PAGE_SIZE;
use crate::device::key::Scancode;
use crate::paging::frames::{PhysAddr, FRAME_ALLOCATOR};

#[derive(Clone, Copy)]
struct FrameBlock {
    addr: PhysAddr,
    frames: usize,
}

/// Test the page frame allocator by performing a few allocations and frees.
pub fn page_frame_demo() {
    println!("Page Frame Demo:");
    println!("");

    println!("Linked list allocator:");
    println!("  Page size: {} bytes", PAGE_SIZE);
    FRAME_ALLOCATOR.lock().dump_free_list();
    println!("");

    println!("Demo 1/5: Allocate single frames");
    println!("");

    let f1 = unsafe {
        FRAME_ALLOCATOR.lock().alloc_block(1).expect("no free physical memory left")
    };
    let f2 = unsafe {
        FRAME_ALLOCATOR.lock().alloc_block(1).expect("no free physical memory left")
    };
    println!("f1: addr={:#x}, frames={}", f1.raw(), 1);
    println!("f2: addr={:#x}, frames={}", f2.raw(), 1);
    println!("");
    FRAME_ALLOCATOR.lock().dump_free_list();
    println!("");
    println!("Press Enter to continue...");
    wait_for_enter();

    println!("Demo 2/5: Free allocated frames");
    println!("---------------------------------------");
    println!("");

    unsafe { FRAME_ALLOCATOR.lock().free_block(f1, 1); }
    unsafe { FRAME_ALLOCATOR.lock().free_block(f2, 1); }
    println!("Freed f1 and f2");
    println!("");
    FRAME_ALLOCATOR.lock().dump_free_list();
    println!("");
    println!("Press Enter to continue...");
    wait_for_enter();

    println!("Demo 3/5: Allocate a block of four frames");
    println!("----------------------------------------------");
    println!("");

    let block = unsafe {
        FRAME_ALLOCATOR.lock().alloc_block(256).expect("no free physical memory left")
    };
    println!("Block capacity: 256 frames ({} bytes)", 256 * PAGE_SIZE);
    println!("Block phys addr: {:#x}", block.raw());
    println!("");
    FRAME_ALLOCATOR.lock().dump_free_list();
    println!("");
    println!("Press Enter to continue...");
    wait_for_enter();

    println!("Demo 4/5: Free allocated block");
    println!("------------------------------");
    println!("");

    unsafe { FRAME_ALLOCATOR.lock().free_block(block,256); }
    println!("Freed block");
    println!("");
    FRAME_ALLOCATOR.lock().dump_free_list();
    println!("");
    println!("Press Enter to continue...");
    wait_for_enter();

    println!("Demo 5/5: Allocate a bunch of blocks");
    println!("------------------------------------");
    println!("");

    let mut pool: Vec<Option<FrameBlock>> = Vec::new();

    for i in 0..201 {
        let frames = (i % 10) + 1;
        let block = FrameBlock { addr: unsafe { FRAME_ALLOCATOR.lock().alloc_block(frames).expect("no free physical memory left") }, frames };
        assert_no_overlap(&pool, block);
        pool.push(Some(block));

        if i % 50 == 0 {
            println!("Allocated blocks: {}", i);
        }
    }

    println!("Initial allocations done");
    println!("");
    FRAME_ALLOCATOR.lock().dump_free_list();
    println!("");
    println!("Press Enter to continue...");
    wait_for_enter();

    for i in 0..pool.len() {
        if i % 3 == 0 {
            if let Some(block) = pool[i] {
                unsafe { FRAME_ALLOCATOR.lock().free_block(block.addr, block.frames); }
            }
            pool[i] = None;
        }
    }

    println!("Partially freed blocks");
    println!("");
    FRAME_ALLOCATOR.lock().dump_free_list();
    println!("");
    println!("Press Enter to continue...");
    wait_for_enter();

    for i in 0..150 {
        let frames = (i % 7) + 2;
        let block = FrameBlock { addr: unsafe { FRAME_ALLOCATOR.lock().alloc_block(frames).expect("no free physical memory left") }, frames };
        assert_no_overlap(&pool, block);
        pool.push(Some(block));

        if i % 50 == 0 {
            println!("Re-allocated blocks: {}", i);
        }
    }

    FRAME_ALLOCATOR.lock().dump_free_list();
    println!("");
    println!("Press Enter to continue...");
    wait_for_enter();

    // final cleanup
    for block in pool.into_iter().flatten() {
        unsafe { FRAME_ALLOCATOR.lock().free_block(block.addr, block.frames); }
    }

    println!("Final state:");
    FRAME_ALLOCATOR.lock().dump_free_list();
    println!("");
    println!("Press Enter to continue...");
    wait_for_enter();
}

fn assert_no_overlap(pool: &[Option<FrameBlock>], block: FrameBlock) {
    let start = block.addr.raw();
    let end = start + block.frames as u64 * PAGE_SIZE as u64;
    for other in pool.iter().flatten() {
        let other_start = other.addr.raw();
        let other_end = other_start + other.frames as u64 * PAGE_SIZE as u64;
        assert!(start >= other_end || other_start >= end, "allocated blocks overlap");
    }
}

fn wait_for_enter() {
    loop {
        let event = crate::device::keyboard::keyboard_buffer().poll_key_press();
        if event.pressed() {
            if let Some(sc) = event.scancode() {
                if sc == Scancode::Enter {
                    break;
                }
            }
        }
    }
}
