use alloc::string::String;
use crate::shell::registry;

pub fn register() {
    registry::register("frames", "Page frame allocator demo", run);
}

fn run(_args: &[String]) {
    crate::demo::lesson10::page_frame_demo();
}
