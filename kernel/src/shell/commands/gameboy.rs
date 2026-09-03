use alloc::string::String;
use crate::shell::registry;

pub fn register() {
    registry::register("gameboy", "Game Boy emulator (usage: gameboy [rom])", run);
}

fn run(args: &[String]) {
    crate::device::terminal::terminal().lock().clear();
    let rom = args.first().map(String::as_str).unwrap_or("roms/pokemon.gb");
    crate::demo::lesson6::peanut_gb::play(rom);
    crate::device::terminal::terminal().lock().clear();
}
