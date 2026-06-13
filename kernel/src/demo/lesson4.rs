/*
 * Contains demos for coroutines and threads.
 *
 * Author: Michael Schoetter, Heinrich Heine University Duesseldorf
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-15
 * License: GPLv3
 */
use log::info;
use crate::coroutine::coroutine::Coroutine;
use crate::device::terminal::terminal;
use crate::thread::scheduler::scheduler;
use crate::thread::thread::Thread;

/// A demo function showcasing coroutines.
/// It starts three coroutines, each incrementing a counter and printing it to the terminal in an endless loop.
/// The coroutines switch to the next coroutine after each print.
pub fn coroutine_demo() {
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

/// A demo function showcasing threads.
/// It starts five threads, each incrementing a counter and printing it to the terminal in an endless loop.
/// The threads yield the CPU to the next thread after each print.
/// The first thread also kills the other four threads after a certain number of iterations and finally exits itself, ending the demo.
pub fn thread_demo() {
    // Initialize scheduler first, so the idle thread takes ID 0
    // Edit: not neccesseray anymore, since first ini in boot.rs
    let sched = scheduler();

    for _ in 0..5 {
        let thread = Thread::new(counter_thread);
        log::info!("Started thread with ID={}", thread.id());
        sched.ready(thread);
    }

    sched.schedule();
}

/// The function executed by each thread in the thread demo.
/// It increments a counter and prints it to the terminal in an endless loop,
/// yielding the CPU to the next thread after each print.
fn counter_thread() {
    let id = scheduler().get_active_tid();
    let mut counter = 0usize;

    loop {
        {
            let mut term = terminal().lock();
            term.set_pos(10, 10 + id);
            print_terminal!(&mut term, "Thread [{}]: counter {}", id, counter);
            drop(term);
        }

        // Thread ID=1 kills the others at different thresholds
        if id == 1 {
            for target in 2..=5 {
                if counter == target * 100 {
                    scheduler().kill(target);
                    {
                        let mut term = terminal().lock();
                        term.set_pos(35, 10 + target);
                        print_terminal!(&mut term, "<Killed by Thread[{}]>", id);
                        drop(term);
                    }
                }
            }

            if counter >= 600 {
                {
                    let mut term = terminal().lock();
                    term.set_pos(35, 10 + id);
                    print_terminal!(&mut term, "<Exited>");
                    drop(term);
                }
                scheduler().exit();
            }
        }

        counter += 1;
        for _ in 0..1000000 {
            // fake sleep
        }

        scheduler().yield_cpu();
    }
}
