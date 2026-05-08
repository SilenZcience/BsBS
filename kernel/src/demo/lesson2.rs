/*
 * Contains a demo for heap allocations.
 *
 * Author: Michael Schoetter, Heinrich Heine University Duesseldorf
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-14
 * License: GPLv3
 */

use alloc::boxed::Box;
use alloc::vec::Vec;
use crate::allocator;
use crate::device::key::Scancode;
use crate::device::keyboard::KEYBOARD;
use crate::device::speaker;
use crate::device::speaker::SPEAKER;
use crate::device::terminal::terminal;

use log::info;

/// A simple heap demo, allocating and freeing memory on the heap.
/// The allocator state is dumped before and after each operation.
pub fn heap_demo() {
    info!("--- heap demo start ---");

    crate::allocator::global::dump_free_list();

    let boxed = Box::new(0xdeadbeefu32);
    info!("Allocated Box at {:p} containing {:#x}", &*boxed, *boxed);

    let mut v: Vec<u8> = Vec::new();
    for i in 0u8..100u8 {
        v.push(i);
        // crate::allocator::global::dump_free_list();
    }
    info!("Allocated Vec at {:p} len={} cap={}", v.as_ptr(), v.len(), v.capacity());

    crate::allocator::global::dump_free_list();
    info!("--- heap demo end ---");
}

/// A demo that plays songs via the PC speaker.
pub fn speaker_demo() {
    todo!("lesson2::speaker_demo() is not implemented yet.")
}
