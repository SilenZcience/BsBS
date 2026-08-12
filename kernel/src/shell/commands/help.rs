use alloc::string::String;
use crate::shell::registry;

pub fn register() {
    registry::register("help", "Show this help message", run);
}

fn run(_args: &[String]) {
    println!("Available commands:");
    for cmd in registry::list() {
        println!("  {:<12} - {}", cmd.name, cmd.help);
    }
}
