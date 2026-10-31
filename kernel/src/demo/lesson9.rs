/*
 * Contains a demo for testing system calls.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-05-28
 * License: GPLv3
 */

use alloc::format;
use alloc::string::String;

use crate::syscall::user_api::{usr_get_key_event, usr_get_system_time, usr_hello_world, usr_print, usr_thread_get_id, usr_thread_wait};
use crate::thread::scheduler::scheduler;
use crate::thread::thread::Thread;

/// Start the syscall demo.
/// It reads keyboard input and prints to the terminal from a user thread.
pub fn syscall_demo() {
    let thread = Thread::new_user_thread(syscall_test_thread);
    let scheduler = scheduler();

    scheduler.ready(thread);
    scheduler.schedule();
}

/// Thread function for the syscall demo.
/// It reads a line from the keyboard, and once the user presses 'Enter',
/// the given line and the current system time are printed to the terminal.
fn syscall_test_thread() {
    usr_print("System Call Demo!\n\n");

    usr_hello_world();

    let system_time = usr_get_system_time();
    usr_print(&format!("System Time: {}\n", system_time));

    let thread_id = usr_thread_get_id();
    usr_print(&format!("Thread ID: {}\n\n", thread_id));

    usr_print("Type 'exit' to exit!\n");
    loop {
        let mut input = String::new();

        usr_print("Enter a line (press Enter to submit): \n");

        loop {
            let key = usr_get_key_event();
            if let Some(key) = key && key.pressed() && key.ascii().is_some() {
                let key_char = key.ascii().unwrap();
                usr_print(&format!("[DBG] Key pressed: {}\n", key_char));
                if key_char == '\r' {
                    break;
                } else {
                    input.push(key_char);
                }
            }
        }

        if input == "exit" {
            usr_print("Exiting syscall demo...\n");
            usr_thread_wait(2000);
            usr_print("Exited syscall demo successfully!\n");
            break;
        }

        usr_print(&format!("You entered: {}\n", input));
    }
}
