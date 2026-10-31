use alloc::string::String;
use core::sync::atomic::Ordering;
use crate::shell::registry;
use crate::thread::scheduler::scheduler;

pub fn register() {
    registry::register("syscall", "Syscall demo", run);
}

fn run(args: &[String]) {
    crate::demo::lesson9::syscall_demo();

    // not sure if the first loop is needed ...
    while !crate::demo::lesson9::DEMO_RUNNING.load(Ordering::SeqCst) {
        scheduler().yield_cpu();
    }
    while crate::demo::lesson9::DEMO_RUNNING.load(Ordering::SeqCst) {
        scheduler().yield_cpu();
    }
}
