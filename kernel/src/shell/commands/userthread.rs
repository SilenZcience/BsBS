use alloc::string::String;
use core::sync::atomic::Ordering;
use crate::shell::registry;
use crate::thread::scheduler::scheduler;

pub fn register() {
    registry::register("userthread", "User thread demo", run);
}

fn run(args: &[String]) {
    crate::device::terminal::terminal().lock().clear();

    crate::demo::lesson8::user_thread_demo();

    // not sure if the first loop is needed ...
    while crate::demo::lesson8::THREADS_RUNNING.load(Ordering::SeqCst) == 0 {
        scheduler().yield_cpu();
    }
    while crate::demo::lesson8::THREADS_RUNNING.load(Ordering::SeqCst) > 0 {
        scheduler().yield_cpu();
    }

    crate::device::terminal::terminal().lock().clear();
}
