use alloc::string::String;
use alloc::vec::Vec;
use crate::device::terminal::terminal;
use crate::shell::readline::read_line;

pub fn run_shell() -> ! {
    println!("HeineOS Shell - Type 'help' for commands");
    loop {
        print!("User@HeineOS:-$ ");
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
    println!("  bitmap      - Bitmap image demo (usage: bitmap [filename])");
    println!("  gameboy     - Game Boy emulator (usage: gameboy [rom])");
}

fn cmd_clear() {
    terminal().lock().clear();
}

fn cmd_uptime() {
    let (hours, minutes, seconds) = crate::device::pit::uptime();
    println!("Uptime: {:02}:{:02}:{:02}", hours, minutes, seconds);
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
    crate::demo::lesson6fs::bitmap_demo(filename);
}

fn cmd_gameboy(args: &[&str]) {
    let rom = args.first().copied().unwrap_or("roms/pokemon.gb");
    crate::demo::lesson6::peanut_gb::play(rom);
    cmd_clear();
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
