use alloc::string::String;
use crate::shell::registry;

pub fn register() {
    registry::register("uptime", "Show system uptime", run);
}

fn run(_args: &[String]) {
    let (hours, minutes, seconds) = crate::device::pit::uptime();
    println!("Uptime: {:02}:{:02}:{:02}", hours, minutes, seconds);
}
