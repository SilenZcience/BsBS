use alloc::string::String;
use crate::library::format::format_size;
use crate::shell::registry;

pub fn register() {
    registry::register("meminfo", "Show memory information", run);
}

fn run(_args: &[String]) {
    let kernel_start = crate::consts::kernel_start();
    let kernel_end = crate::consts::kernel_end();
    println!("Kernel data segment:");
    println!("  Start: 0x{:x}", kernel_start);
    println!("  End:   0x{:x}", kernel_end);
    println!("  Size:  {}", format_size(kernel_end - kernel_start));
    println!("");

    let heap_start = crate::consts::heap_start();
    let heap_end = heap_start + crate::consts::HEAP_SIZE;
    println!("Kernel heap:");
    println!("  Start: 0x{:x}", heap_start);
    println!("  End  : 0x{:x}", heap_end);

    let stats = crate::allocator::global::heap_stats();
    println!("  Total: {}", format_size(stats.total));
    println!("  Used : {}", format_size(stats.used));
    println!("  Free : {}", format_size(stats.free));
    println!("  Free blocks: {}", stats.free_blocks);
    println!("  Largest free block: {}", format_size(stats.largest_free_block));
    println!("");

    println!("Physical memory (UEFI memory map):");
    match crate::sysinfo::memory_stats() {
        Some(stats) => {
            println!("  Total  : {}", format_size(stats.total as usize));
            println!("  Usable : {}", format_size(stats.usable as usize));
            println!("  Entries: {}", stats.entries);
        }
        None => {
            println!("  (memory map not available)");
        }
    }
}
