use alloc::string::String;
use crate::shell::registry;

pub fn register() {
    registry::register("bitmap", "Bitmap image demo (usage: bitmap [filename] [x] [y])", run);
}

fn run(args: &[String]) {
    let filename = args.first().map(String::as_str).unwrap_or("img/heine.bmp");
    let x = args.get(1).and_then(|s| s.parse::<usize>().ok());
    let y = args.get(2).and_then(|s| s.parse::<usize>().ok());
    crate::demo::lesson6fs::bitmap_demo(filename, x, y);
}
