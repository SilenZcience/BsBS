use alloc::string::String;
use crate::shell::registry;

pub fn register() {
    registry::register("text", "Text formatting demo", run);
}

fn run(_args: &[String]) {
    crate::demo::lesson1::text_demo();
}
