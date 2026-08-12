use alloc::string::String;
use crate::shell::registry;

pub fn register() {
    registry::register("clear", "Clear the screen", run);
}

fn run(_args: &[String]) {
    crate::device::terminal::terminal().lock().clear();
}
