use alloc::string::String;
use uefi::mem::memory_map::{MemoryMap, MemoryMapOwned, MemoryType};
use crate::library::once::Once;

static BOOTLOADER_NAME: Once<String> = Once::new();

static MEMORY_MAP: Once<MemoryMapOwned> = Once::new();


pub fn set_bootloader_name(name: &str) {
    BOOTLOADER_NAME.init(|| String::from(name));
}

pub fn bootloader_name() -> Option<&'static str> {
    BOOTLOADER_NAME.get().map(|name| name.as_str())
}

pub fn set_memory_map(memory_map: MemoryMapOwned) {
    MEMORY_MAP.init(|| memory_map);
}

/// summary of memory layout reported by the UEFI memory map
pub struct MemoryStats {
    /// amount of physical memory in bytes
    pub total: u64,
    /// amount of free memory in bytes
    pub usable: u64,
    /// amount of descriptors in the memory map
    pub entries: usize,
}

pub fn memory_stats() -> Option<MemoryStats> {
    let map = MEMORY_MAP.get()?;

    let mut total = 0u64;
    let mut usable = 0u64;

    for desc in map.entries() {
        let bytes = desc.page_count * 4096;
        total += bytes;
        if desc.ty == MemoryType::CONVENTIONAL {
            usable += bytes;
        }
    }

    Some(MemoryStats {
        total,
        usable,
        entries: map.len(),
    })
}
