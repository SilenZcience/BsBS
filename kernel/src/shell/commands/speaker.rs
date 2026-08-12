use alloc::string::String;
use crate::shell::registry;

pub fn register() {
    registry::register("speaker", "PC speaker demo (usage: speaker [tetris|aerodynamic])", run);
}

fn run(args: &[String]) {
    crate::demo::lesson2::speaker_demo(args);
}
