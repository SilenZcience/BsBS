/*
 * Contains demos for coroutines and threads.
 *
 * Author: Michael Schoetter, Heinrich Heine University Duesseldorf
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-15
 * License: GPLv3
 */
use log::info;
use crate::coroutine::coroutine::Coroutine;
use crate::device::pit;
use crate::device::speaker::tetris;
use crate::device::terminal::terminal;
use crate::thread::scheduler::scheduler;
use crate::thread::thread::Thread;

/// A demo function showcasing coroutines.
/// It starts three coroutines, each incrementing a counter and printing it to the terminal in an endless loop.
/// The coroutines switch to the next coroutine after each print.
/// the coroutines run on their own thread...
pub fn coroutine_demo() {
    let thread = Thread::new(coroutine_demo_loop);
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
    let mut counter = 0usize;
    loop {
        let mut term = terminal().lock();
        term.set_pos(10, 10 + coroutine.id());
        print_terminal!(&mut term, "Coroutine [{}]: {}", coroutine.id(), counter);
        drop(term);
        counter += 1;
        for _ in 0..1000000 {
            // fake sleep
        }
        coroutine.switch();
    }
}

/// A demo function showcasing preemptive multithreading.
/// It starts three counter threads and one speaker thread.
/// The PIT drives thread switching every 10ms.
pub fn thread_demo() {
    let sched = scheduler();

    for _ in 0..5 {
        let thread = Thread::new(counter_thread);
        info!("Started thread with ID={}", thread.id());
        sched.ready(thread);
    }

    // let speaker = Thread::new(speaker_thread);
    // info!("Started speaker thread with ID={}", speaker.id());
    // sched.ready(speaker);

}

/// The function executed by each counter thread.
/// It increments a counter and prints it to the terminal,
/// then waits 100ms to give the PIT a chance to switch threads.
fn counter_thread() {
    let id = scheduler().get_active_tid();
    let mut counter = 0usize;
    let start_time = pit::system_time();

    loop {
        {
            let mut term = terminal().lock();
            term.set_pos(10, 10 + id);
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
                term.set_pos(35, 10 + id);
                print_terminal!(&mut term, "<Exited (Finished after {}ms)>", elapsed);
                drop(term);
            }
            scheduler().exit();
        }
    }
}

fn speaker_thread() {
    let start_time = pit::system_time();
    tetris();

    let elapsed = pit::system_time() - start_time;
    let id = scheduler().get_active_tid();
    let mut term = terminal().lock();
    term.set_pos(35, 10 + id);
    print_terminal!(&mut term, "<Speaker Exited (Finished after {}ms)>", elapsed);
    drop(term);

    scheduler().exit();
}
