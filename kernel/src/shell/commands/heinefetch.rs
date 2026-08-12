use alloc::string::String;
use crate::device::cpu;
use crate::library::format::format_size;
use crate::shell::registry;

pub fn register() {
    registry::register("heinefetch", "Show system information", run);
}

fn run(_args: &[String]) {
    let bootloader = crate::sysinfo::bootloader_name().unwrap_or("unknown");
    let (hours, minutes, seconds) = crate::device::pit::uptime();
    let (fb_w, fb_h) = {
        let fb = crate::device::terminal::framebuffer().lock();
        (fb.width(), fb.height())
    };
    let (cols, rows) = crate::device::terminal::terminal().lock().size();
    let stats = crate::allocator::global::heap_stats();
    println!("                   *** ### ### ***                    User@HeineOS");
    println!("               *##                 ##*                ------------");
    println!("           *##                         ##*            OS:         HeineOS 0.1.0 (x86_64)");
    println!("        *##                               ##*         Bootloader: {}", bootloader);
    println!("      *##                                   ##*       Uptime:     {:02}h {:02}m {:02}s", hours, minutes, seconds);
    println!("    *##                                       ##*     Shell:      HeineOS Shell v0.1.0");
    println!("   *##                                         ##*    Resolution: {}x{}", fb_w, fb_h);
    println!("  *##                                           ##*   Terminal:   {}x{} characters", cols, rows);
    println!(" *##         @@      @@                          ##*  Vendor:     {}", cpu::vendor());
    println!(" *##         @@      @@                          ##*  CPU:        {}", cpu::brand());
    println!(" *##         @@      @@                          ##*  Memory:     {} / {}", format_size(stats.used), format_size(stats.total));
    println!(" *##         @@@@@@  @@@@@@  @@  @@              ##*");
    println!(" *##         @@  @@  @@  @@  @@  @@              ##*");
    println!(" *##         @@  @@  @@  @@  @@  @@              ##*");
    println!(" *##         @@  @@  @@  @@  @@@@@@  @@@         ##*");
    println!("  *##                                @@@        ##*");
    println!("   *##                                         ##*");
    println!("    *##                                       ##*");
    println!("      *##                                   ##*");
    println!("        *##                               ##*");
    println!("           *##                         ##*");
    println!("               *##                 ##*");
    println!("                   *** ### ### ***");
    println!("");
}
