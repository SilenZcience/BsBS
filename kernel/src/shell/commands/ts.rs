use alloc::string::{String, ToString};
use crate::shell::registry;

pub fn register() {
    registry::register("ts", "Show thread snapshot information", run);
}

fn run(_args: &[String]) {
    let (active, ready, terminated) = crate::thread::scheduler::scheduler().thread_ids();

    println!("Thread information:");
    println!("  Active thread:  T{}", active);
    println!("  Threads ready:  {}", format_ids(&ready));
    println!("  Threads exited: {}", format_ids(&terminated));
}

fn format_ids(ids: &[usize]) -> String {
    if ids.is_empty() {
        return String::from("(none)");
    }

    let mut result = String::new();
    for (i, id) in ids.iter().enumerate() {
        if i > 0 {
            result.push(' ');
        }
        result.push('T');
        result.push_str(&id.to_string());
    }
    result
}
