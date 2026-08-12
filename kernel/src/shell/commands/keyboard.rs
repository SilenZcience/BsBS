use alloc::string::String;
use crate::shell::registry;

pub fn register() {
    registry::register("keyboard", "Keyboard event demo", run);
}

fn run(_args: &[String]) {
    crate::demo::lesson1::keyboard_demo();
}
