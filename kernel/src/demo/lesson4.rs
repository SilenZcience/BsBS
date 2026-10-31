/*
 * Contains demos for coroutines and threads.
 *
 * Author: Michael Schoetter, Heinrich Heine University Duesseldorf
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-15
 * License: GPLv3
 */
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use log::info;
use crate::coroutine::coroutine::Coroutine;
use crate::device::pit;
use crate::device::speaker::tetris;
use crate::device::terminal::terminal;
use crate::thread::scheduler::scheduler;
use crate::thread::thread::Thread;

pub static COROUTINES_RUNNING: AtomicBool = AtomicBool::new(false);
pub static THREADS_RUNNING: AtomicUsize = AtomicUsize::new(0);
const NUM_THREADS: usize = 5;

static NEXT_THREAD_ROW: AtomicUsize = AtomicUsize::new(0);
static NEXT_COROUTINE_ROW: AtomicUsize = AtomicUsize::new(0);

/// A demo function showcasing coroutines.
/// It starts three coroutines, each incrementing a counter and printing it to the terminal in an endless loop.
/// The coroutines switch to the next coroutine after each print.
/// the coroutines run on their own thread...
pub fn coroutine_demo() {
    COROUTINES_RUNNING.store(true, Ordering::SeqCst);
    NEXT_COROUTINE_ROW.store(0, Ordering::SeqCst);

    let thread = Thread::new_kernel_thread(coroutine_demo_loop);
    info!("Started thread with ID={}", thread.id());
    scheduler().ready(thread);
}

fn coroutine_demo_loop() {
    let mut c1 = Coroutine::new(coroutine_loop);
    let mut c2 = Coroutine::new(coroutine_loop);
    let mut c3 = Coroutine::new(coroutine_loop);
    let mut c4 = Coroutine::new(coroutine_loop);
    let mut c5 = Coroutine::new(coroutine_loop);

    c1.set_next(&mut *c2);
    c2.set_next(&mut *c3);
    c3.set_next(&mut *c4);
    c4.set_next(&mut *c5);
    c5.set_next(&mut *c1);

    c1.start();
}

/// The function executed by each coroutine in the coroutine demo.
/// It increments a counter and prints it to the terminal in an endless loop,
/// switching to the next coroutine after each print.
fn coroutine_loop(coroutine: &mut Coroutine) {
    let id = coroutine.id();
    let row = NEXT_COROUTINE_ROW.fetch_add(1, Ordering::SeqCst);
    let mut counter = 1usize;
    let start_time = pit::system_time();
    loop {
        if pit::system_time() - start_time >= 5000 {
            COROUTINES_RUNNING.store(false, Ordering::SeqCst);
            scheduler().exit();
        }

        let mut term = terminal().lock();
        term.set_pos(10, 10 + row);
        print_terminal!(&mut term, "Coroutine [{}]: {}", id, counter);
        drop(term);
        counter += 1;
        // for _ in 0..1000000 {
        //     // fake sleep
        // }
        pit::wait(200);
        coroutine.switch();
    }
}

/// A demo function showcasing preemptive multithreading.
/// It starts three counter threads and one speaker thread.
/// The PIT drives thread switching every 10ms.
pub fn thread_demo() {
    THREADS_RUNNING.store(NUM_THREADS, Ordering::SeqCst);
    NEXT_THREAD_ROW.store(0, Ordering::SeqCst);

    let sched = scheduler();

    for _ in 0..5 {
        let thread = Thread::new_kernel_thread(counter_thread);
        info!("Started thread with ID={}", thread.id());
        sched.ready(thread);
    }
}

/// The function executed by each counter thread.
/// It increments a counter and prints it to the terminal
fn counter_thread() {
    let id = scheduler().get_active_tid();
    let row = NEXT_THREAD_ROW.fetch_add(1, Ordering::SeqCst);
    let mut counter = 0usize;
    let start_time = pit::system_time();

    loop {
        {
            let mut term = terminal().lock();
            term.set_pos(10, 10 + row);
            print_terminal!(&mut term, "Thread [{}]: counter {}", id, counter);
            drop(term);
        }

        if counter % 10 == 0 {
            scheduler().yield_cpu();
        }

        counter += 1;
        if counter >= 1000 {
            let elapsed = pit::system_time() - start_time;
            {
                let mut term = terminal().lock();
                term.set_pos(35, 10 + row);
                print_terminal!(&mut term, "<Exited (Finished after {}ms)>", elapsed);
                drop(term);
            }
            THREADS_RUNNING.fetch_sub(1, Ordering::SeqCst);
            scheduler().exit();
        }
    }
}

fn speaker_thread() {
    let start_time = pit::system_time();
    tetris();

    let elapsed = pit::system_time() - start_time;
    let row = NEXT_THREAD_ROW.fetch_add(1, Ordering::SeqCst);
    let mut term = terminal().lock();
    term.set_pos(35, 10 + row);
    print_terminal!(&mut term, "<Speaker Exited (Finished after {}ms)>", elapsed);
    drop(term);

    scheduler().exit();
}
