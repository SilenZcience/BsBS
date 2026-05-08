/*
 * Contains demos for text output and keyboard input.
 *
 * Author: Michael Schoetter, Heinrich Heine University Duesseldorf
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-14
 * License: GPLv3
 */
/// A simple text demo, displaying formatted numbers.
pub fn text_demo() {
    println!("Text Demo:");
    println!("    dec | hex |   bin   |");
    println!("  |-----|-----|---------|");
    for i in 0..32 {
        println!("  | {:>3} | {:>3x} | {:>7b} |", i, i, i);
    }
}

/// A simple keyboard demo, displaying the events of key presses and releases.
pub fn keyboard_demo() {
    use crate::device::keyboard::KEYBOARD;
    println!("Keyboard Demo: Press keys to see events. Press ESC to exit.");
    loop {
        let event = KEYBOARD.lock().poll_key_event();
        let ascii = match event.ascii() {
            Some(c) => c,
            None => '\''
        };
        let pressed = event.pressed();
        println!(
            "KeyEvent: ascii: '{}' scancode: {:?} modifiers: {:?} pressed: {}",
            ascii,
            event.scancode(),
            event.modifiers(),
            pressed
        );
        // Exit on ESC key press
        if let Some(sc) = event.scancode() {
            if sc == crate::device::key::Scancode::Escape && event.pressed() {
                break;
            }
        }
    }
}
