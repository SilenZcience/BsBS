use alloc::string::String;
use core::sync::atomic::Ordering;
use crate::device::pit;
use crate::shell::registry;
use crate::thread::scheduler::scheduler;

pub fn register() {
    registry::register("thread", "Thread demo", run);
}

fn run(_args: &[String]) {
    crate::device::terminal::terminal().lock().clear();

    crate::demo::lesson4::thread_demo();

    // not sure if the first loop is needed ...
    while crate::demo::lesson4::THREADS_RUNNING.load(Ordering::SeqCst) == 0 {
        scheduler().yield_cpu();
    }
    while crate::demo::lesson4::THREADS_RUNNING.load(Ordering::SeqCst) > 0 {
        scheduler().yield_cpu();
    }

    pit::wait(2500);
    crate::device::terminal::terminal().lock().clear();
}
