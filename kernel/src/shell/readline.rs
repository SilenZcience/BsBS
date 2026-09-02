use alloc::string::String;
use alloc::vec::Vec;
use crate::device::key::Scancode;
use crate::device::keyboard::keyboard_buffer;
use crate::device::terminal::{terminal, DEFAULT_BG_COLOR, INPUT_COLOR};
use crate::library::once::Once;
use crate::library::spinlock::Spinlock;
use crate::thread::scheduler::scheduler;
use crate::shell::shell::print_prompt;

const MAX_HISTORY: usize = 64;

struct History {
    entries: Vec<String>,
    index: usize,
    saved: String,
}

static HISTORY: Once<Spinlock<History>> = Once::new();

fn history() -> &'static Spinlock<History> {
    HISTORY.init(|| {
        Spinlock::new(History {
            entries: Vec::new(),
            index: 0,
            saved: String::new(),
        })
    })
}

fn history_up(buf: &mut String) {
    let mut hist = history().lock();
    if hist.entries.is_empty() {
        return;
    }
    if hist.index == hist.entries.len() {
        hist.saved = buf.clone();
    }
    if hist.index == 0 {
        return;
    }
    hist.index -= 1;

    let mut term = terminal().lock();
    for _ in 0..buf.len() {
        term.backspace();
    }
    *buf = hist.entries[hist.index].clone();
    for c in buf.chars() {
        term.put_char_colored(c, INPUT_COLOR, DEFAULT_BG_COLOR);
    }
}

fn history_down(buf: &mut String) {
    let mut hist = history().lock();
    if hist.index >= hist.entries.len() {
        return;
    }

    let mut term = terminal().lock();
    for _ in 0..buf.len() {
        term.backspace();
    }

    hist.index += 1;
    if hist.index == hist.entries.len() {
        *buf = core::mem::take(&mut hist.saved);
    } else {
        *buf = hist.entries[hist.index].clone();
    }
    for c in buf.chars() {
        term.put_char_colored(c, INPUT_COLOR, DEFAULT_BG_COLOR);
    }
}

pub fn read_line(mut cmd_names: &mut Vec<String>) -> String {
    let mut buf = String::new();
    loop {
        // no busy-spinning.
        let event = loop {
            if let Some(key) = keyboard_buffer().pop_key_event() {
                if key.pressed() {
                    break key;
                }
            } else {
                scheduler().yield_cpu();
            }
        };

        if let Some(c) = event.ascii() {
            match c {
                '\r' => {
                    println!("");
                    if !buf.is_empty() {
                        let mut hist = history().lock();
                        if hist.entries.last().map_or(true, |last| *last != buf) {
                            if hist.entries.len() >= MAX_HISTORY {
                                hist.entries.remove(0);
                            }
                            hist.entries.push(buf.clone());
                        }
                        hist.index = hist.entries.len();
                        hist.saved.clear();
                    }
                    return buf;
                }
                '\x08' => {
                    if buf.pop().is_some() {
                        terminal().lock().backspace();
                    }
                }
                c if c.is_ascii_graphic() || c == ' ' => {
                    buf.push(c);
                    let byte = [c as u8];
                    print_colored!(core::str::from_utf8(&byte).unwrap(), INPUT_COLOR);
                }
                _ => {}
            }
        } else if event.scancode() == Some(Scancode::Tab) {
            auto_complete(&mut cmd_names, &mut buf);
        } else if event.scancode() == Some(Scancode::Down) {
            history_down(&mut buf);
        } else if event.scancode() == Some(Scancode::Up) {
            history_up(&mut buf);
        }
    }
}

fn auto_complete(cmd_names: &mut Vec<String>, buf: &mut String) {
    let rword_start = buf.rfind(' ').map_or(0, |i| i + 1);
    let rword = &buf[rword_start..];

    let files = crate::filesystem::tarfs::filesystem().list_files();
    let matches: Vec<&String> = if rword_start == 0 {
        cmd_names
            .iter()
            .filter(|name| name.starts_with(rword))
            .collect()
    } else {
        files
            .iter()
            .map(|(name, _)| name)
            .filter(|name| name.starts_with(rword))
            .collect()
    };

    if matches.is_empty() {
        return;
    } else if matches.len() > 1 {
        let mut match_width = matches[0].len();
        for name in &matches[1..] {
            match_width = match_width.max(name.len());
        }
        match_width += 2;
        let (cols, _) = terminal().lock().size();
        let per_line = 1.max(cols / match_width);
        let mut count = 1;
        println!("");
        for name in &matches {
            print!("{:<w$}", name, w = match_width);
            if count % per_line == 0 {
                println!("");
            }
            count += 1;
        }
        if count % per_line != 1 {
            println!("");
        }
        print_prompt();
        for c in buf.chars() {
            let byte = [c as u8];
            print_colored!(core::str::from_utf8(&byte).unwrap(), INPUT_COLOR);
        }
    }

    let mut longest_commong_prefix = matches[0].clone();
    for name in &matches[1..] {
        let mut i = 0;
        while i < longest_commong_prefix.len() && i < name.len() && longest_commong_prefix.as_bytes()[i] == name.as_bytes()[i] {
            i += 1;
        }
        longest_commong_prefix.truncate(i);
    }

    if longest_commong_prefix.len() > rword.len() {
        for c in longest_commong_prefix[rword.len()..].chars() {
            let byte = [c as u8];
            print_colored!(core::str::from_utf8(&byte).unwrap(), INPUT_COLOR);
            buf.push(c);
        }

        if matches.len() == 1 {
            buf.push(' ');
            print!(" ");
        }
    }
}
