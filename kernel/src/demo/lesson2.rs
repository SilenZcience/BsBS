/*
 * Contains a demo for heap allocations.
 *
 * Author: Michael Schoetter, Heinrich Heine University Duesseldorf
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-14
 * License: GPLv3
 */

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::mem::size_of;
use log::info;
use crate::device::key::Scancode;
use crate::device::speaker;
use crate::thread::scheduler::scheduler;
use crate::thread::thread::Thread;

/// A simple heap demo, allocating and freeing memory on the heap.
pub fn heap_demo() {
    // NOTE: change allocator back to bumpallocator in global.rs
    // println!("Heap Demo:");
    // println!("");

    // crate::allocator::global::dump_free_list();
    // println!("");

    // let boxed = Box::new(0xdeadbeefu32);
    // println!("Allocated Box at {:p} containing {:#x}", &*boxed, *boxed);

    // let mut v: Vec<u8> = Vec::new();
    // for i in 0u8..100u8 {
    //     v.push(i);
    //     // crate::allocator::global::dump_free_list();
    // }
    // println!("Allocated Vec at {:p} len={} cap={}", v.as_ptr(), v.len(), v.capacity());
    // println!("");

    // crate::allocator::global::dump_free_list();
    // println!("Heap Demo End");
    // println!("");
    println!("Heap Demo:");
    println!("");

    println!("Linked list allocator:");
    println!(
        "  Heap start: 0x{:x}, Heap end: 0x{:x}",
        crate::consts::heap_start(),
        crate::consts::heap_start() + crate::consts::HEAP_SIZE
    );

    crate::allocator::global::dump_free_list();
    println!("");

    println!("Demo 1/5: Allocate structs using 'Box'");
    println!("");

    let s1 = Box::new(MyStruct { a: 1, b: 2 });
    let s2 = Box::new(MyStruct { a: 3, b: 4 });
    let s1_ptr = (&*s1) as *const MyStruct as usize;
    let s2_ptr = (&*s2) as *const MyStruct as usize;
    println!("s1 = {{ a: {}, b: {} }}", s1.a, s1.b);
    println!("s2 = {{ a: {}, b: {} }}", s2.a, s2.b);
    println!(
        "s1 ptr: {:#x}, s2 ptr: {:#x}, sizeof(MyStruct)={}B",
        s1_ptr,
        s2_ptr,
        size_of::<MyStruct>()
    );
    println!("");

    println!("Linked list allocator:");
    println!(
        "  Heap start: 0x{:x}, Heap end: 0x{:x}",
        crate::consts::heap_start(),
        crate::consts::heap_start() + crate::consts::HEAP_SIZE
    );
    crate::allocator::global::dump_free_list();
    println!("");
    println!("Press Enter to continue...");
    wait_for_enter();

    println!("Demo 2/5: Free allocated structs");
    println!("---------------------------------------");
    println!("");

    drop(s1);
    drop(s2);
    println!("Dropped s1 and s2");
    println!("");

    println!("Linked list allocator:");
    crate::allocator::global::dump_free_list();
    println!("");
    println!("Press Enter to continue...");
    wait_for_enter();

    println!("Demo 3/5: Allocate a Vec of three structs");
    println!("---------------------------------------");
    println!("");

    let mut vec: Vec<MyStruct> = Vec::with_capacity(4);
    vec.push(MyStruct { a: 5, b: 6 });
    vec.push(MyStruct { a: 7, b: 8 });
    vec.push(MyStruct { a: 9, b: 0 });

    println!(
        "Vec capacity: {} elements ({} bytes)",
        vec.capacity(),
        vec.capacity() * size_of::<MyStruct>()
    );
    println!("Vec data ptr: {:#x}", vec.as_ptr() as usize);

    println!("vec[0] = {{ a: {}, b: {} }}", vec[0].a, vec[0].b);
    println!("vec[1] = {{ a: {}, b: {} }}", vec[1].a, vec[1].b);
    println!("vec[2] = {{ a: {}, b: {} }}", vec[2].a, vec[2].b);
    println!("");

    println!("Linked list allocator:");
    crate::allocator::global::dump_free_list();
    println!("");
    println!("Press Enter to continue...");
    wait_for_enter();

    println!("Demo 4/5: Free allocated Vec");
    println!("---------------------------------------");
    println!("");

    drop(vec);
    println!("Dropped Vec");
    println!("");

    println!("Linked list allocator:");
    crate::allocator::global::dump_free_list();
    println!("");
    println!("Press Enter to continue...");
    wait_for_enter();

    println!("Demo 5/5: Allocate a bunch of objects");
    println!("---------------------------------------");
    println!("");

    let mut pool: Vec<Option<Vec<MyStruct>>> = Vec::new();

    for i in 0..201 {
        let mut v: Vec<MyStruct> = Vec::with_capacity((i % 10) + 1);

        for j in 0..v.capacity() {
            v.push(MyStruct {
                a: i as usize,
                b: j as usize,
            });
        }

        pool.push(Some(v));

        if i % 50 == 0 {
            println!("Allocated Vecs: {}", i);
        }
    }

    println!("Initial allocations done");
    println!("");

    println!("Linked list allocator:");
    crate::allocator::global::dump_free_list();
    println!("");

    println!("Press Enter to continue...");
    wait_for_enter();

    for i in 0..pool.len() {
        if i % 3 == 0 {
            pool[i] = None; // drop Vec
        }
    }

    println!("Partially freed Vecs");
    println!("");

    println!("Linked list allocator:");
    crate::allocator::global::dump_free_list();
    println!("");

    println!("Press Enter to continue...");
    wait_for_enter();

    for i in 0..150 {
        let mut v: Vec<MyStruct> = Vec::with_capacity((i % 7) + 2);

        for j in 0..v.capacity() {
            v.push(MyStruct {
                a: (i * 10) as usize,
                b: j as usize,
            });
        }

        pool.push(Some(v));

        if i % 50 == 0 {
            println!("Re-allocated Vecs: {}", i);
        }
    }

    println!("Linked list allocator:");
    crate::allocator::global::dump_free_list();
    println!("");

    println!("Press Enter to continue...");
    wait_for_enter();

    // final cleanup
    for slot in pool {
        drop(slot);
    }

    println!("Final state:");
    println!("Linked list allocator:");
    crate::allocator::global::dump_free_list();
    println!("");

    println!("Press Enter to continue...");
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

/// A demo that plays songs via the PC speaker.
pub fn speaker_demo(args: &[String]) {
    let song = args.first().map(String::as_str).unwrap_or("tetris");

    let entry = match song {
        "aerodynamic" => play_aerodynamic,
        _ => play_tetris,
    };

    let mut thread = Thread::new(entry);
    thread.set_kill_handler(speaker::stop);
    info!("Started speaker thread with ID={}", thread.id());
    scheduler().ready(thread);
}

fn play_tetris() {
    println!("Speaker Demo: Playing Tetris theme...");
    speaker::tetris();
    println!("Finished!");
}

fn play_aerodynamic() {
    println!("Speaker Demo: Playing Aerodynamic...");
    speaker::aerodynamic();
    println!("Finished!");
}
