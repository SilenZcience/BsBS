use crate::device::{key::KeyEvent, keyboard::keyboard_buffer, terminal::{self, terminal}};


pub extern "C" fn sys_print(message_ptr: *const u8, message_len: usize) {
    let msg = unsafe {
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(message_ptr, message_len))
    };

    if msg.is_empty() {
        return;
    }

    let mut terminal = terminal().lock();
    terminal::print(&mut terminal, format_args!("{}", msg));
}

pub extern "C" fn sys_get_key_event(key_event_ptr: *mut KeyEvent) -> u64 {
    let k_event = keyboard_buffer().pop_key_event();

    if let Some(event) = k_event {
        unsafe {
            core::ptr::write(key_event_ptr, event);
        }
        0 // Success
    } else {
        1 // No key event available
    }

}
