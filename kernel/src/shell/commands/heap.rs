use alloc::string::String;
use crate::shell::registry;

pub fn register() {
    registry::register("heap", "Heap allocator demo", run);
}

fn run(_args: &[String]) {
    crate::demo::lesson2::heap_demo();
}
