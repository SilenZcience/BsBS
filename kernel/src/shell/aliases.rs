use alloc::string::String;
use alloc::vec::Vec;
use crate::alloc::string::ToString;
use crate::library::once::Once;
use crate::library::spinlock::Spinlock;
use log::info;

const ALIASES_FILE: &str = ".aliases.txt";

pub const MAX_ALIAS_EXPANSION: usize = 16;

// (name, value) pairs
static ALIASES: Once<Spinlock<Vec<(String, String)>>> = Once::new();

fn aliases() -> &'static Spinlock<Vec<(String, String)>> {
    ALIASES.init(|| Spinlock::new(Vec::new()))
}

// parse ALIASES_FILE
pub fn load_aliases() {
    let fs = crate::filesystem::tarfs::filesystem();
    if let Ok(handle) = fs.open(ALIASES_FILE) {
        let size = fs.size(handle).unwrap_or(0);
        let mut buf = alloc::vec![0u8; size];
        if fs.read(handle, &mut buf).is_ok() {
            if let Ok(text) = core::str::from_utf8(&buf) {
                let mut list = aliases().lock();
                for line in text.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Some(eq) = line.find('=') {
                        let name = line[..eq].trim();
                        let value = line[eq + 1..].trim();
                        if !name.is_empty() && !value.is_empty() {
                            list.push((String::from(name), String::from(value)));
                            info!("Loaded alias: {} -> {}", name, value);
                        }
                    }
                }
            }
        }
        let _ = fs.close(handle);
    }
}

fn lookup_alias(name: &str) -> Option<String> {
    aliases()
        .lock()
        .iter()
        .find(|(alias, _)| alias == name)
        .map(|(_, value)| value.clone())
}

pub fn expand_aliases(command: &str) -> String {
    let mut current_command = command;
    let mut expanded: Option<String> = None;
    for _ in 0..MAX_ALIAS_EXPANSION {
        match lookup_alias(current_command) {
            Some(value) => {
                expanded = Some(value);
                current_command = expanded.as_deref().unwrap();
                info!("Expanded alias: {} -> {}", command, current_command);
            }
            None => break,
        }
    }
    expanded.as_deref().unwrap_or(current_command).to_string()
}
