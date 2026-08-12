use alloc::string::String;
use crate::shell::registry;

pub fn register() {
    registry::register("speaker", "PC speaker demo", run);
}

fn run(_args: &[String]) {
    crate::demo::lesson2::speaker_demo();
}
