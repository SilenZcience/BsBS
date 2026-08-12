use alloc::string::String;
use crate::shell::registry;

pub fn register() {
    registry::register("thread", "Thread demo", run);
}

fn run(_args: &[String]) {
    crate::demo::lesson4::thread_demo();
}
