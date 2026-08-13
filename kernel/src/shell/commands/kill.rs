use alloc::string::String;
use crate::shell::registry;

pub fn register() {
    registry::register("kill", "Kill a thread by ID (usage: kill <tid>)", run);
}

fn run(args: &[String]) {
    let Some(arg) = args.first() else {
        println!("Usage: kill <tid>");
        return;
    };

    let Ok(id) = arg.parse::<usize>() else {
        println!("Invalid thread ID: '{}'", arg);
        return;
    };

    if crate::thread::scheduler::scheduler().kill(id) {
        println!("Killed thread T{}", id);
    } else {
        println!("Thread T{} not found", id);
    }
}
