use alloc::string::String;
use crate::shell::registry;

pub fn register() {
    registry::register("coroutine", "Coroutine demo", run);
}

fn run(_args: &[String]) {
    crate::demo::lesson4::coroutine_demo();
}
