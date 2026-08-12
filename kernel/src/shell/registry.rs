use alloc::string::String;
use alloc::vec::Vec;
use crate::library::once::Once;
use crate::library::spinlock::Spinlock;

pub type CommandFn = fn(&[String]);

#[derive(Clone, Copy)]
pub struct Command {
    pub name: &'static str,
    pub help: &'static str,
    pub run: CommandFn,
}

impl Command {
    pub const fn new(name: &'static str, help: &'static str, run: CommandFn) -> Self {
        Command { name, help, run }
    }
}

static REGISTRY: Once<Spinlock<Vec<Command>>> = Once::new();

fn registry() -> &'static Spinlock<Vec<Command>> {
    REGISTRY.init(|| Spinlock::new(Vec::new()))
}

pub fn register(name: &'static str, help: &'static str, run: CommandFn) {
    registry().lock().push(Command::new(name, help, run));
}

pub fn find(name: &str) -> Option<Command> {
    registry().lock().iter().find(|cmd| cmd.name == name).copied()
}

pub fn list() -> Vec<Command> {
    registry().lock().clone()
}
