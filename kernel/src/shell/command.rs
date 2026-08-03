use alloc::string::String;
use alloc::vec::Vec;
use crate::device::terminal::{terminal, PROMPT_COLOR};
use crate::shell::readline::read_line;

pub fn run_shell() -> ! {
    println!("HeineOS Shell - Type 'help' for commands");
    loop {
        print_colored!("User@HeineOS:-$ ", PROMPT_COLOR);
        let line = read_line();
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        let cmd = parts[0];
        let args = &parts[1..];

        match cmd {
            "clear" => cmd_clear(),
            "uptime" => cmd_uptime(),
            "heinefetch" => cmd_uname(),
            "meminfo" => cmd_meminfo(),
            "cpuinfo" => cmd_cpuinfo(),
            "ps" => cmd_ps(),
            "ls" => cmd_ls(),
            "echo" => cmd_echo(args),
            "help" => cmd_help(),
            "text" => cmd_text(),
            "keyboard" => cmd_keyboard(),
            "heap" => cmd_heap(),
            "speaker" => cmd_speaker(),
            "coroutine" => cmd_coroutine(),
            "thread" => cmd_thread(),
            "cat" => cmd_textfile(args),
            "bitmap" => cmd_bitmap(args),
            "gameboy" => cmd_gameboy(args),
            "reboot" => cmd_reboot(),
            "shutdown" => cmd_shutdown(),
            _ => {
                println!("Unknown command: '{}'", cmd);
                println!("Type 'help' for available commands");
            }
        }
    }
}

fn cmd_help() {
    println!("Available commands:");
    println!("  help        - Show this help message");
    println!("  clear       - Clear the screen");
    println!("  uptime      - Show system uptime");
    println!("  heinefetch  - Show system information");
    println!("  meminfo     - Show memory information");
    println!("  cpuinfo     - Show CPU information");
    println!("  ps          - Show thread information");
    println!("  ls          - List files in the initrd");
    println!("  echo        - Print text to the terminal (usage: echo [text])");
    println!("  text        - Text formatting demo");
    println!("  keyboard    - Keyboard event demo");
    println!("  heap        - Heap allocator demo");
    println!("  speaker     - PC speaker demo");
    println!("  coroutine   - Coroutine demo");
    println!("  thread      - Thread demo");
    println!("  cat         - Display text file contents (usage: cat [filename])");
    println!("  bitmap      - Bitmap image demo (usage: bitmap [filename] [x] [y])");
    println!("  gameboy     - Game Boy emulator (usage: gameboy [rom])");
    println!("  reboot      - Reboot the system");
    println!("  shutdown    - Power off the system");
}

fn cmd_clear() {
    terminal().lock().clear();
}

fn cmd_uptime() {
    let (hours, minutes, seconds) = crate::device::pit::uptime();
    println!("Uptime: {:02}:{:02}:{:02}", hours, minutes, seconds);
}

fn cmd_uname() {
    terminal().lock().clear();

    let x_pos = 54;
    let mut y_pos = 2;

    println!("User@HeineOS:-$ heinefetch");
    println!(" ");

    terminal().lock().set_pos(x_pos, y_pos);
    y_pos += 1;
    println!("User@HeineOS");

    terminal().lock().set_pos(x_pos, y_pos);
    y_pos += 1;
    println!("------------");

    terminal().lock().set_pos(x_pos, y_pos);
    y_pos += 1;
    println!("OS:         HeineOS 0.1.0 (x86_64)");

    terminal().lock().set_pos(x_pos, y_pos);
    y_pos += 1;
    let bootloader = crate::sysinfo::bootloader_name().unwrap_or("unknown");
    println!("Bootloader: {}", bootloader);

    terminal().lock().set_pos(x_pos, y_pos);
    y_pos += 1;
    let (hours, minutes, seconds) = crate::device::pit::uptime();
    println!("Uptime:     {:02}h {:02}m {:02}s", hours, minutes, seconds);

    terminal().lock().set_pos(x_pos, y_pos);
    y_pos += 1;
    println!("Shell:      HeineOS Shell v0.1.0");

    terminal().lock().set_pos(x_pos, y_pos);
    y_pos += 1;
    let (fb_w, fb_h) = {
        let fb = crate::device::terminal::framebuffer().lock();
        (fb.width(), fb.height())
    };
    println!("Resolution: {}x{}", fb_w, fb_h);

    terminal().lock().set_pos(x_pos, y_pos);
    y_pos += 1;
    let (cols, rows) = terminal().lock().size();
    println!("Terminal:   {}x{} characters", cols, rows);

    terminal().lock().set_pos(x_pos, y_pos);
    y_pos += 1;
    println!("Vendor:     {}", cpu_vendor());

    terminal().lock().set_pos(x_pos, y_pos);
    y_pos += 1;
    println!("CPU:        {}", cpu_brand());

    terminal().lock().set_pos(x_pos, y_pos);
    // y_pos += 1;
    let stats = crate::allocator::global::heap_stats();
    println!("Memory:     {} / {}", format_size(stats.used), format_size(stats.total));

    // cmd_bitmap(&["img/hhu.bmp", "0", "25"]);
    // terminal().lock().set_pos(0, 27);
    terminal().lock().set_pos(0, 2);
    println!("                   *** ### ### ***");
    println!("               *##                 ##*");
    println!("           *##                         ##*");
    println!("        *##                               ##*");
    println!("      *##                                   ##*");
    println!("    *##                                       ##*");
    println!("   *##                                         ##*");
    println!("  *##                                           ##*");
    println!(" *##         @@      @@                          ##*");
    println!(" *##         @@      @@                          ##*");
    println!(" *##         @@      @@                          ##*");
    println!(" *##         @@@@@@  @@@@@@  @@  @@              ##*");
    println!(" *##         @@  @@  @@  @@  @@  @@              ##*");
    println!(" *##         @@  @@  @@  @@  @@  @@              ##*");
    println!(" *##         @@  @@  @@  @@  @@@@@@  @@@         ##*");
    println!("  *##                                @@@        ##*");
    println!("   *##                                         ##*");
    println!("    *##                                       ##*");
    println!("      *##                                   ##*");
    println!("        *#                                ##*");
    println!("           *##                         ##*");
    println!("               *##                 ##*");
    println!("                   *** ### ### ***");
    println!("");
}

fn cmd_meminfo() {
    let kernel_start = crate::consts::kernel_start();
    let kernel_end   = crate::consts::kernel_end();
    println!("Kernel data segment:");
    println!("  Start: 0x{:x}", kernel_start);
    println!("  End:   0x{:x}", kernel_end  );
    println!("  Size:  {}"    , format_size(kernel_end - kernel_start));
    println!("");

    let heap_start = crate::consts::heap_start();
    let heap_end = heap_start + crate::consts::HEAP_SIZE;
    println!("Kernel heap:");
    println!("  Start: 0x{:x}", heap_start);
    println!("  End  : 0x{:x}", heap_end);

    let stats = crate::allocator::global::heap_stats();
    println!("  Total: {}", format_size(stats.total));
    println!("  Used : {}", format_size(stats.used ));
    println!("  Free : {}", format_size(stats.free ));
    println!("  Free blocks: {}", stats.free_blocks );
    println!("  Largest free block: {}", format_size(stats.largest_free_block));
    println!("");

    println!("Physical memory (UEFI memory map):");
    match crate::sysinfo::memory_stats() {
        Some(stats) => {
            println!("  Total  : {}", format_size(stats.total  as usize));
            println!("  Usable : {}", format_size(stats.usable as usize));
            println!("  Entries: {}", stats.entries);
        }
        None => {
            println!("  (memory map not available)");
        }
    }
}


fn cmd_cpuinfo() {
    println!("CPU Information");
    println!("---------------");

    println!("Vendor:            {}", cpu_vendor());

    let (eax, ebx, ecx, edx) = crate::device::cpu::cpuid(1);
    let family = ((eax >> 8) & 0xf) + ((eax >> 20) & 0xff);
    let model = ((eax >> 4) & 0xf) + (((eax >> 16) & 0xf) << 4);
    let stepping = eax & 0xf;
    let logical_cpus = ((ebx >> 16) & 0xff) as usize;
    println!("Family/Model/Step: {}/{}/{}", family, model, stepping);
    println!("Logical CPUs:      {}", logical_cpus);

    println!("Model name:        {}", cpu_brand());

    println!("Features:");
    let mut features: Vec<&str> = Vec::new();
    if ecx & (1 <<  0) != 0 { features.push("sse3"             ); }
    if ecx & (1 <<  1) != 0 { features.push("pclmulqdq"        ); }
    if ecx & (1 <<  2) != 0 { features.push("dtes64"           ); }
    if ecx & (1 <<  3) != 0 { features.push("monitor"          ); }
    if ecx & (1 <<  4) != 0 { features.push("ds-cpl"           ); }
    if ecx & (1 <<  5) != 0 { features.push("vmx"              ); }
    if ecx & (1 <<  6) != 0 { features.push("smx"              ); }
    if ecx & (1 <<  7) != 0 { features.push("eist"             ); }
    if ecx & (1 <<  8) != 0 { features.push("tm2"              ); }
    if ecx & (1 <<  9) != 0 { features.push("ssse3"            ); }
    if ecx & (1 << 10) != 0 { features.push("cnxt-id"          ); }
    if ecx & (1 << 11) != 0 { features.push("sdbg"             ); }
    if ecx & (1 << 12) != 0 { features.push("fma"              ); }
    if ecx & (1 << 13) != 0 { features.push("cmpxchg16b"       ); }
    if ecx & (1 << 14) != 0 { features.push("xtprupdatecontrol"); }
    if ecx & (1 << 15) != 0 { features.push("pdcm"             ); }
    if ecx & (1 << 17) != 0 { features.push("pcid"             ); }
    if ecx & (1 << 18) != 0 { features.push("dca"              ); }
    if ecx & (1 << 19) != 0 { features.push("sse4.1"           ); }
    if ecx & (1 << 20) != 0 { features.push("sse4.2"           ); }
    if ecx & (1 << 21) != 0 { features.push("x2apic"           ); }
    if ecx & (1 << 22) != 0 { features.push("movbe"            ); }
    if ecx & (1 << 23) != 0 { features.push("popcnt"           ); }
    if ecx & (1 << 24) != 0 { features.push("tsc-deadline"     ); }
    if ecx & (1 << 25) != 0 { features.push("aesni"            ); }
    if ecx & (1 << 26) != 0 { features.push("xsave"            ); }
    if ecx & (1 << 27) != 0 { features.push("osxsave"          ); }
    if ecx & (1 << 28) != 0 { features.push("avx"              ); }
    if ecx & (1 << 29) != 0 { features.push("f16c"             ); }
    if ecx & (1 << 30) != 0 { features.push("rdrand"           ); }

    if edx & (1 <<  0) != 0 { features.push("fpu"   ); }
    if edx & (1 <<  1) != 0 { features.push("vme"   ); }
    if edx & (1 <<  2) != 0 { features.push("de"    ); }
    if edx & (1 <<  3) != 0 { features.push("pse"   ); }
    if edx & (1 <<  4) != 0 { features.push("tsc"   ); }
    if edx & (1 <<  5) != 0 { features.push("msr"   ); }
    if edx & (1 <<  6) != 0 { features.push("pae"   ); }
    if edx & (1 <<  7) != 0 { features.push("mce"   ); }
    if edx & (1 <<  8) != 0 { features.push("cx8"   ); }
    if edx & (1 <<  9) != 0 { features.push("apic"  ); }
    if edx & (1 << 11) != 0 { features.push("sep"   ); }
    if edx & (1 << 12) != 0 { features.push("mtrr"  ); }
    if edx & (1 << 13) != 0 { features.push("pge"   ); }
    if edx & (1 << 14) != 0 { features.push("mca"   ); }
    if edx & (1 << 15) != 0 { features.push("cmov"  ); }
    if edx & (1 << 16) != 0 { features.push("pat"   ); }
    if edx & (1 << 17) != 0 { features.push("pse-36"); }
    if edx & (1 << 18) != 0 { features.push("psn"   ); }
    if edx & (1 << 19) != 0 { features.push("clfsh" ); }
    if edx & (1 << 21) != 0 { features.push("ds"    ); }
    if edx & (1 << 22) != 0 { features.push("acpi"  ); }
    if edx & (1 << 23) != 0 { features.push("mmx"   ); }
    if edx & (1 << 24) != 0 { features.push("fxsr"  ); }
    if edx & (1 << 25) != 0 { features.push("sse"   ); }
    if edx & (1 << 26) != 0 { features.push("sse2"  ); }
    if edx & (1 << 27) != 0 { features.push("ss"    ); }
    if edx & (1 << 28) != 0 { features.push("htt"   ); }
    if edx & (1 << 29) != 0 { features.push("tm"    ); }
    if edx & (1 << 31) != 0 { features.push("pbe"   ); }

    if features.is_empty() {
        println!("  (none reported)");
    } else {
        let mut first = true;
        for feature in features {
            if !first {
                print!(" ");
            }
            print!("{}", feature);
            first = false;
        }
        println!("");
    }
}

fn cpu_brand() -> String {
    let (max_ext, _, _, _) = crate::device::cpu::cpuid(0x8000_0000);
    if max_ext < 0x8000_0004 {
        return String::from("unknown");
    }

    let mut brand = String::new();
    for leaf in 0x8000_0002..=0x8000_0004 {
        let (a, b, c, d) = crate::device::cpu::cpuid(leaf);
        for reg in [a, b, c, d] {
            for byte in reg.to_le_bytes() {
                if byte == 0 {
                    return String::from(brand.trim());
                }
                brand.push(byte as char);
            }
        }
    }
    String::from(brand.trim())
}

fn cpu_vendor() -> String {
    let (_, ebx, ecx, edx) = crate::device::cpu::cpuid(0);
    let mut vendor = String::new();
    vendor.push_str(core::str::from_utf8(&ebx.to_le_bytes()).unwrap_or("?"));
    vendor.push_str(core::str::from_utf8(&edx.to_le_bytes()).unwrap_or("?"));
    vendor.push_str(core::str::from_utf8(&ecx.to_le_bytes()).unwrap_or("?"));
    vendor
}

fn cmd_ps() {
    let (active, ready, terminated) = crate::thread::scheduler::scheduler().thread_stats();
    println!("Thread information:");
    println!("  Active thread:  T{}", active    );
    println!("  Threads ready:  {}" , ready     );
    println!("  Threads exited: {}" , terminated);
}

fn cmd_ls() {
    let files = crate::filesystem::tarfs::filesystem().list_files();

    if files.is_empty() {
        println!("(initrd is empty)");
        return;
    }

    for (name, size) in &files {
        println!("  {:<40} {:>10} B {:>13}", name, size, format_size(*size));
    }

    let total: usize = files.iter().map(|(_, size)| size).sum();
    println!("");
    println!("{} file(s), total {}", files.len(), format_size(total));
}

fn cmd_echo(args: &[&str]) {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    println!("");
}

fn cmd_text() {
    crate::demo::lesson1::text_demo();
}

fn cmd_keyboard() {
    crate::demo::lesson1::keyboard_demo();
}

fn cmd_heap() {
    crate::demo::lesson2::heap_demo();
}

fn cmd_speaker() {
    crate::demo::lesson2::speaker_demo();
}

fn cmd_coroutine() {
    crate::demo::lesson4::coroutine_demo();
}

fn cmd_thread() {
    crate::demo::lesson4::thread_demo();
}

fn cmd_textfile(args: &[&str]) {
    let filename = args.first().copied().unwrap_or("lorem.txt");
    crate::demo::lesson6fs::text_file_demo(filename);
}

fn cmd_bitmap(args: &[&str]) {
    let filename = args.first().copied().unwrap_or("img/heine.bmp");
    let x = args.get(1).and_then(|s| s.parse::<usize>().ok());
    let y = args.get(2).and_then(|s| s.parse::<usize>().ok());
    crate::demo::lesson6fs::bitmap_demo(filename, x, y);
}

fn cmd_gameboy(args: &[&str]) {
    let rom = args.first().copied().unwrap_or("roms/pokemon.gb");
    crate::demo::lesson6::peanut_gb::play(rom);
    cmd_clear();
}

fn cmd_reboot() {
    uefi::runtime::reset(uefi::runtime::ResetType::COLD, uefi::Status::SUCCESS, None);
}

fn cmd_shutdown() {
    uefi::runtime::reset(uefi::runtime::ResetType::SHUTDOWN, uefi::Status::SUCCESS, None);
}

fn format_size(size_bytes: usize) -> String {
    const SIZE_NAME: [&str; 9] = [" B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];

    if size_bytes == 0 {
        return String::from("0  B");
    }

    let mut i = 0;
    let mut power = 1usize;
    while power <= size_bytes / 1024 && i < SIZE_NAME.len() - 1 {
        power *= 1024;
        i += 1;
    }

    let whole = size_bytes / power;
    let hundredths = ((size_bytes % power) as u128 * 100 + (power / 2) as u128) / power as u128;

    if hundredths == 100 {
        alloc::format!("{} {}", whole + 1, SIZE_NAME[i])
    } else {
        alloc::format!("{}.{:02} {}", whole, hundredths, SIZE_NAME[i])
    }
}
