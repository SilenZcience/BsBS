use alloc::string::String;
use crate::shell::registry;

pub fn register() {
    registry::register("cat", "Display text file contents (usage: cat [filename])", run);
}

fn run(args: &[String]) {
    let filename = args.first().map(String::as_str).unwrap_or("lorem.txt");
    crate::demo::lesson6fs::text_file_demo(filename);
}
