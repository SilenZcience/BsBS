use alloc::string::String;
use alloc::vec::Vec;
use crate::device::key::Scancode;
use crate::device::keyboard::keyboard_buffer;
use crate::device::terminal::terminal;
use crate::library::once::Once;
use crate::library::spinlock::Spinlock;

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
        term.put_char(c);
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
        term.put_char(c);
    }
}

pub fn read_line() -> String {
    let mut buf = String::new();
    loop {
        let event = keyboard_buffer().poll_key_press();

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
                    print!("{}", c);
                }
                _ => {}
            }
        } else if event.scancode() == Some(Scancode::Down) {
            history_down(&mut buf);
        } else if event.scancode() == Some(Scancode::Up) {
            history_up(&mut buf);
        }
    }
}
