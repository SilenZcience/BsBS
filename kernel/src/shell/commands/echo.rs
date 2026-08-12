use alloc::string::String;
use crate::shell::registry;

pub fn register() {
    registry::register("echo", "Print text to the terminal (usage: echo [text])", run);
}

fn run(args: &[String]) {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    println!("");
}
