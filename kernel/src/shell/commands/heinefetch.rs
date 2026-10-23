use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use crate::device::cpu;
use crate::library::format::format_size;
use crate::shell::registry;

const INFO_COL: usize  = 54;
const VALUE_COL: usize = 66;

const LOGO: [&str; 23] = [
    "                   *** ### ### ***",
    "               *##                 ##*",
    "           *##                         ##*",
    "        *##                               ##*",
    "      *##                                   ##*",
    "    *##                                       ##*",
    "   *##                                         ##*",
    "  *##                                           ##*",
    " *##         @@      @@                          ##*",
    " *##         @@      @@                          ##*",
    " *##         @@      @@                          ##*",
    " *##         @@@@@@  @@@@@@  @@  @@              ##*",
    " *##         @@  @@  @@  @@  @@  @@              ##*",
    " *##         @@  @@  @@  @@  @@  @@              ##*",
    " *##         @@  @@  @@  @@  @@@@@@  @@@         ##*",
    "  *##                                @@@        ##*",
    "   *##                                         ##*",
    "    *##                                       ##*",
    "      *##                                   ##*",
    "        *##                               ##*",
    "           *##                         ##*",
    "               *##                 ##*",
    "                   *** ### ### ***",
];

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

    let uptime     = format!("{:02}h {:02}m {:02}s", hours, minutes, seconds);
    let resolution = format!("{}x{}", fb_w, fb_h);
    let terminal   = format!("{}x{} characters", cols, rows);
    let memory     = format!("{} / {}", format_size(stats.used), format_size(stats.total));

    let mut info: Vec<String> = Vec::new();
    info.push(String::from("kernel@HeineOS"));
    info.push(String::from("--------------"));
    push_entry(&mut info, "OS:", "HeineOS 0.1.0 (x86_64)", cols);
    push_entry(&mut info, "Bootloader:", bootloader, cols);
    push_entry(&mut info, "Uptime:", &uptime, cols);
    push_entry(&mut info, "Shell:", "HeineOS Shell v0.1.0", cols);
    push_entry(&mut info, "Resolution:", &resolution, cols);
    push_entry(&mut info, "Terminal:", &terminal, cols);
    push_entry(&mut info, "Vendor:", &cpu::vendor(), cols);
    push_entry(&mut info, "CPU:", &cpu::brand(), cols);
    push_entry(&mut info, "Memory:", &memory, cols);

    for (i, art) in LOGO.iter().enumerate() {
        match info.get(i) {
            Some(line) => println!("{:<width$}{}", art, line, width = INFO_COL),
            None => println!("{}", art),
        }
    }
    for line in info.iter().skip(LOGO.len()) {
        println!("{:<width$}{}", "", line, width = INFO_COL);
    }
    println!("");
}

fn push_entry(info: &mut Vec<String>, label: &str, value: &str, cols: usize) {
    let label = format!("{:<width$}", label, width = VALUE_COL - INFO_COL);
    let indent = " ".repeat(VALUE_COL - INFO_COL);
    let width = cols.saturating_sub(VALUE_COL).max(1);

    let mut rest = value;
    let mut first = true;

    loop {
        let (head, tail) = split_at_limit(rest, width);

        if first {
            info.push(format!("{}{}", label, head));
            first = false;
        } else {
            info.push(format!("{}{}", indent, head));
        }

        if tail.is_empty() {
            break;
        }

        rest = tail;
    }
}

fn split_at_limit(s: &str, limit: usize) -> (&str, &str) {
    let width = limit.max(1);

    let mut count = 0;
    let mut space = None;
    let mut end = None;

    for (i, c) in s.char_indices() {
        if count == width {
            end = Some(i);
            break;
        }
        if c == ' ' {
            space = Some(i);
        }
        count += 1;
    }

    let end = match end {
        Some(end) => end,
        None => return (s, ""),
    };

    let cut = match space {
        Some(i) if i > 0 => i,
        _ => end,
    };

    (&s[..cut], s[cut..].trim_start())
}
