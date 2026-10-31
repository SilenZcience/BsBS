/*
 * Module: user_api
 *
 * Description: All system calls available to user programs are defined in this module.
 *
 * Author: Stefan Lankes, RWTH Aachen University
 *         Licensed under the Apache License, Version 2.0 or MIT license, at your option.
 *
 *         Michael Schoettner, Heinrich Heine University Duesseldorf, 14.09.2023
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 15.10.2025
 */

use core::arch::asm;

use crate::device::key::KeyEvent;

/// System call numbers available to user programs.
#[repr(u64)]
pub enum SyscallFunction {
    HelloWorld,
    ThreadYield,
    ThreadWait,
    ThreadExit,
    ThreadGetId,
    GetSystemTime,
    Print,
    GetKeyEvent,
    NumSyscalls // Last entry to count number of syscalls
}

/// Test system call printing "Hello, World!" to the serial console.
pub fn usr_hello_world() {
    unsafe { syscall0(SyscallFunction::HelloWorld); }
}

/// Yields CPU & switches to next thread
pub fn usr_thread_yield() {
    unsafe { syscall0(SyscallFunction::ThreadYield); }
}

/// yields cpu for ms milliseconds
pub fn usr_thread_wait(ms: usize) {
    unsafe {
        syscall1(SyscallFunction::ThreadWait, ms as u64);
    }
}

/// kill current thread
pub fn usr_thread_exit() {
    unsafe { syscall0(SyscallFunction::ThreadExit); }
}

/// get the ID of the current thread
pub fn usr_thread_get_id() -> u64 {
    unsafe { syscall0(SyscallFunction::ThreadGetId) }
}

/// get the system time
pub fn usr_get_system_time() -> u64 {
    unsafe { syscall0(SyscallFunction::GetSystemTime) }
}

/// print a string to the serial console
pub fn usr_print(s: &str) {
    unsafe {
        syscall2(SyscallFunction::Print, s.as_ptr() as u64, s.len() as u64);
    }
}

/// get a key event from the keyboard
pub fn usr_get_key_event() -> Option<KeyEvent> {
    let mut key_event = KeyEvent::new();
    let ret = unsafe {
        syscall1(SyscallFunction::GetKeyEvent, core::ptr::from_mut(&mut key_event) as u64)
    };
    if ret == 0 {
        Some(key_event)
    } else {
        None
    }
}

/// Perform a system call with 0 parameters.
#[inline(always)]
pub unsafe fn syscall0(syscall: SyscallFunction) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!(
        "int 0x80",
        inlateout("rax") syscall as u64 => ret,
        options(preserves_flags, nostack)
        );
    }
    ret
}

/// perform a system call with 1 parameter
#[inline(always)]
pub unsafe fn syscall1(syscall: SyscallFunction, arg1: u64) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!(
        "int 0x80",
        inlateout("rax") syscall as u64 => ret,
        in("rdi") arg1,
        options(preserves_flags, nostack)
        );
    }
    ret
}

/// perform a system call with 2 parameters
#[inline(always)]
pub unsafe fn syscall2(syscall: SyscallFunction, arg1: u64, arg2: u64) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!(
        "int 0x80",
        inlateout("rax") syscall as u64 => ret,
        in("rdi") arg1,
        in("rsi") arg2,
        options(preserves_flags, nostack)
        );
    }
    ret
}

/// perform a system call with 3 parameters
#[inline(always)]
pub unsafe fn syscall3(syscall: SyscallFunction, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!(
        "int 0x80",
        inlateout("rax") syscall as u64 => ret,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        options(preserves_flags, nostack)
        );
    }
    ret
}

/// perform a system call with 4 parameters
#[inline(always)]
pub unsafe fn syscall4(syscall: SyscallFunction, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!(
        "int 0x80",
        inlateout("rax") syscall as u64 => ret,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("rcx") arg4,
        options(preserves_flags, nostack)
        );
    }
    ret
}

/// perform a system call with 5 parameters
#[inline(always)]
pub unsafe fn syscall5(syscall: SyscallFunction, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!(
        "int 0x80",
        inlateout("rax") syscall as u64 => ret,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("rcx") arg4,
        in("r8") arg5,
        options(preserves_flags, nostack)
        );
    }
    ret
}
