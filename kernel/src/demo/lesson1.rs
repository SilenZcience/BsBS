/*
 * Contains demos for text output and keyboard input.
 *
 * Author: Michael Schoetter, Heinrich Heine University Duesseldorf
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-14
 * License: GPLv3
 */

use crate::device::terminal::terminal;

/// A simple text demo, displaying formatted numbers.
pub fn text_demo() {
    use core::fmt::Write;
    let mut term = terminal().lock();
    writeln!(term, "Text Demo:").unwrap();
    writeln!(term, "    dec | hex |   bin   |").unwrap();
    writeln!(term, "  |-----|-----|---------|").unwrap();
    for i in 0..32 {
        writeln!(
            term,
            "  | {:>3} | {:>3x} | {:>7b} |",
            i, i, i
        ).unwrap();
    }
}

/// A simple keyboard demo, displaying the events of key presses and releases.
pub fn keyboard_demo() {
    use crate::device::keyboard::KEYBOARD;
    use core::fmt::Write;
    let mut term = terminal().lock();
    writeln!(term, "Keyboard Demo: Press keys to see events. Press ESC to exit.").unwrap();
    loop {
        let event = KEYBOARD.lock().poll_key_event();
        let ascii = match event.ascii() {
            Some(c) => c,
            None => '\''
        };
        let pressed = event.pressed();
        write!(term, "KeyEvent: ascii: '{}' scancode: ", ascii).unwrap();
        match event.scancode() {
            Some(code) => write!(term, "{:?}", code).unwrap(),
            None => write!(term, "None").unwrap(),
        }
        write!(term, " modifiers: {:?} pressed: {}\n", event.modifiers(), pressed).unwrap();
        // Exit on ESC key press
        if let Some(sc) = event.scancode() {
            if sc == crate::device::key::Scancode::Escape && event.pressed() {
                break;
            }
        }
    }
}
