use alloc::string::String;
use crate::shell::registry;

pub fn register() {
    registry::register("ps", "Show thread information", run);
}

fn run(_args: &[String]) {
    let (active, ready, terminated) = crate::thread::scheduler::scheduler().thread_stats();
    println!("Thread information:");
    println!("  Active thread:  T{}", active);
    println!("  Threads ready:  {}", ready);
    println!("  Threads exited: {}", terminated);
}
