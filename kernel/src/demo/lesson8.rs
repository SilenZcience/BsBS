use core::sync::atomic::{AtomicUsize, Ordering};

use crate::device::pit;
use crate::device::terminal::terminal;
use crate::thread::scheduler::scheduler;
use crate::thread::thread::Thread;

pub static THREADS_RUNNING: AtomicUsize = AtomicUsize::new(0);
const NUM_THREADS: usize = 3;

const SLEEP_DELAY: usize = 100;

pub fn user_thread_demo() {
    THREADS_RUNNING.store(NUM_THREADS, Ordering::SeqCst);

    let k1 = Thread::new_kernel_thread(kernel_thread);
    let u1 = Thread::new_user_thread(user_thread);
    let u2 = Thread::new_user_thread(user_thread);

    let scheduler = scheduler();
    scheduler.ready(k1);
    scheduler.ready(u1);
    scheduler.ready(u2);

    scheduler.yield_cpu();
}

fn thread_finished() {
    THREADS_RUNNING.fetch_sub(1, Ordering::SeqCst);
}

fn kernel_thread() {
    let id = scheduler().get_active_tid();
    let cols = terminal().lock().size().0;

    for i in 0..cols - 1 {
        {
            let mut terminal = terminal().lock();
            terminal.set_pos(i, id);
            terminal.put_char('K');
        }

        pit::wait(SLEEP_DELAY);
    }

    thread_finished();
}

fn user_thread() {
    let id = scheduler().get_active_tid();
    let cols = terminal().lock().size().0;

    for i in 0..cols - 1 {
        {
            let mut terminal = terminal().lock();
            terminal.set_pos(i, id);
            terminal.put_char('U');
        }

        // User threads may not yield the CPU manually and thus cannot call pit::wait(),
        // as it calls Scheduler::yield_cpu() internally.
        // Instead, we implement a simple busy-wait loop here.
        let start = pit::system_time();
        while pit::system_time() - start < SLEEP_DELAY {
            core::hint::spin_loop();
        }
    }

    thread_finished();
}
