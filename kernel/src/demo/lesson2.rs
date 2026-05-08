/*
 * Contains a demo for heap allocations.
 *
 * Author: Michael Schoetter, Heinrich Heine University Duesseldorf
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-14
 * License: GPLv3
 */

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::fmt::Write;
use core::mem::size_of;
use crate::device::terminal::terminal;
use crate::device::keyboard::KEYBOARD;
use crate::device::key::Scancode;
use crate::device::speaker;
use crate::device::speaker::SPEAKER;

/// A simple heap demo, allocating and freeing memory on the heap.
pub fn heap_demo() {
    // let mut term = terminal().lock(); // NOTE: change allocator back to bumpallocator in global.rs

    // writeln!(term, "Heap Demo:").unwrap();
    // writeln!(term).unwrap();

    // crate::allocator::global::dump_free_list(&mut term);
    // writeln!(term).unwrap();

    // let boxed = Box::new(0xdeadbeefu32);
    // writeln!(term, "Allocated Box at {:p} containing {:#x}", &*boxed, *boxed).unwrap();

    // let mut v: Vec<u8> = Vec::new();
    // for i in 0u8..100u8 {
    //     v.push(i);
    //     // crate::allocator::global::dump_free_list(&mut term);
    // }
    // writeln!(term, "Allocated Vec at {:p} len={} cap={}", v.as_ptr(), v.len(), v.capacity()).unwrap();
    // writeln!(term).unwrap();

    // crate::allocator::global::dump_free_list(&mut term);
    // writeln!(term, "Heap Demo End").unwrap();
    // writeln!(term).unwrap();
    let mut term = terminal().lock();

    writeln!(term, "Heap Demo:").unwrap();
    writeln!(term).unwrap();

    writeln!(term, "Linked list allocator:").unwrap();
    writeln!(term, "  Heap start: 0x{:x}, Heap end: 0x{:x}", crate::consts::heap_start(), crate::consts::heap_start() + crate::consts::HEAP_SIZE).unwrap();
    crate::allocator::global::dump_free_list(&mut term);
    writeln!(term).unwrap();

    writeln!(term, "Demo 1/4: Allocate structs using 'Box'").unwrap();
    writeln!(term).unwrap();

    let s1 = Box::new(MyStruct { a: 1, b: 2 });
    let s2 = Box::new(MyStruct { a: 3, b: 4 });
    let s1_ptr = (&*s1) as *const MyStruct as usize;
    let s2_ptr = (&*s2) as *const MyStruct as usize;
    writeln!(term, "s1 = {{ a: {}, b: {} }}", s1.a, s1.b).unwrap();
    writeln!(term, "s2 = {{ a: {}, b: {} }}", s2.a, s2.b).unwrap();
    writeln!(term, "s1 ptr: {:#x}, s2 ptr: {:#x}, sizeof(MyStruct)={}B", s1_ptr, s2_ptr, size_of::<MyStruct>()).unwrap();
    writeln!(term).unwrap();

    writeln!(term, "Linked list allocator:").unwrap();
    writeln!(term, "  Heap start: 0x{:x}, Heap end: 0x{:x}", crate::consts::heap_start(), crate::consts::heap_start() + crate::consts::HEAP_SIZE).unwrap();
    crate::allocator::global::dump_free_list(&mut term);
    writeln!(term).unwrap();
    writeln!(term, "Press Enter to continue...").unwrap();
    drop(term);
    wait_for_enter();

    let mut term = terminal().lock();
    writeln!(term, "Demo 2/4: Free allocated structs").unwrap();
    writeln!(term, "---------------------------------------").unwrap();
    writeln!(term).unwrap();

    drop(s1);
    drop(s2);
    writeln!(term, "Dropped s1 and s2").unwrap();
    writeln!(term).unwrap();

    writeln!(term, "Linked list allocator:").unwrap();
    crate::allocator::global::dump_free_list(&mut term);
    writeln!(term).unwrap();
    writeln!(term, "Press Enter to continue...").unwrap();
    drop(term);
    wait_for_enter();

    let mut term = terminal().lock();
    writeln!(term, "Demo 3/4: Allocate a Vec of three structs").unwrap();
    writeln!(term, "---------------------------------------").unwrap();
    writeln!(term).unwrap();

    let mut vec: Vec<MyStruct> = Vec::with_capacity(4);
    vec.push(MyStruct { a: 5, b: 6 });
    vec.push(MyStruct { a: 7, b: 8 });
    vec.push(MyStruct { a: 9, b: 0 });

    writeln!(term, "Vec capacity: {} elements ({} bytes)", vec.capacity(), vec.capacity() * size_of::<MyStruct>()).unwrap();
    writeln!(term, "Vec data ptr: {:#x}", vec.as_ptr() as usize).unwrap();

    writeln!(term, "vec[0] = {{ a: {}, b: {} }}", vec[0].a, vec[0].b).unwrap();
    writeln!(term, "vec[1] = {{ a: {}, b: {} }}", vec[1].a, vec[1].b).unwrap();
    writeln!(term, "vec[2] = {{ a: {}, b: {} }}", vec[2].a, vec[2].b).unwrap();
    writeln!(term).unwrap();

    writeln!(term, "Linked list allocator:").unwrap();
    crate::allocator::global::dump_free_list(&mut term);
    writeln!(term).unwrap();
    writeln!(term, "Press Enter to continue...").unwrap();
    drop(term);
    wait_for_enter();

    let mut term = terminal().lock();
    writeln!(term, "Demo 4/4: Free allocated Vec").unwrap();
    writeln!(term, "---------------------------------------").unwrap();
    writeln!(term).unwrap();

    drop(vec);
    writeln!(term, "Dropped Vec").unwrap();
    writeln!(term).unwrap();

    writeln!(term, "Linked list allocator:").unwrap();
    crate::allocator::global::dump_free_list(&mut term);
    writeln!(term).unwrap();
    writeln!(term, "Press Enter to continue...").unwrap();
    drop(term);
    wait_for_enter();
}

/// A simple struct for testing heap allocations
#[derive(Clone, Copy)]
struct MyStruct {
    a: usize,
    b: usize,
}

fn wait_for_enter() {
    loop {
        let event = KEYBOARD.lock().poll_key_event();
        if event.pressed() {
            if let Some(sc) = event.scancode() {
                if sc == Scancode::Enter {
                    break;
                }
            }
        }
    }
}

/// A demo that plays songs via the PC speaker.
pub fn speaker_demo() {
    todo!("lesson2::speaker_demo() is not implemented yet.")
}
