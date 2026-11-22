/*
 * All system calls are routed here via the IDT syscall handler (interrupt 0x80).
 * The system call number is passed in the rax register and is used as an index into
 * the syscall table to call the corresponding function.
 *
 * Author: Stefan Lankes, RWTH Aachen University
 *         Licensed under the Apache License, Version 2.0 or MIT license, at your option.
 *
 *         Michael Schoettner, Heinrich Heine University Duesseldorf, 2024-10-23
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2025-10-15
 */

use core::arch::naked_asm;
use crate::syscall::functions::hello::sys_hello_world;
use crate::syscall::functions::io::{sys_print, sys_get_key_event};
use crate::syscall::functions::system::sys_get_system_time;
use crate::syscall::functions::thread::{sys_thread_exit, sys_thread_get_id, sys_thread_wait, sys_thread_yield};
use crate::syscall::user_api::SyscallFunction;

/// Global syscall function table.
static SYSCALL_TABLE: SyscallFunctionTable = SyscallFunctionTable::new();

/// Struct to hold the syscall function pointers.
#[repr(align(64))]
#[repr(C)]
struct SyscallFunctionTable {
    table: [*const u64; SyscallFunction::NumSyscalls as usize],
}

impl SyscallFunctionTable {
    pub const fn new() -> SyscallFunctionTable {
        SyscallFunctionTable {
            table: [
                sys_hello_world as *const u64,
                sys_thread_yield as *const u64,
                sys_thread_wait as *const u64,
                sys_thread_exit as *const u64,
                sys_thread_get_id as *const u64,
                sys_get_system_time as *const u64,
                sys_print as *const u64,
                sys_get_key_event as *const u64,
            ],
        }
    }
}

unsafe impl Send for SyscallFunctionTable {}
unsafe impl Sync for SyscallFunctionTable {}

/// System call dispatcher.
/// This function is called from the IDT syscall handler (interrupt 0x80).
/// The syscall number is passed in the rax register and is used as an index into
/// the syscall table to call the corresponding function.
/// All registers (except rax) are saved and restored.
/// The syscall function's return value is passed back in rax.
#[unsafe(naked)]
pub extern "C" fn dispatch_syscall() {
    naked_asm!(
        // Save all registers (except rax, which contains the syscall number)
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "push rbx",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push rbp",

        // Call syscall handler (or syscall_abort for an invalid syscall number)
        "cmp rax, {NUM_SYSCALLS}",
        "jae syscall_abort",
        "call [{SYSCALL_TABLE} + rax * 8]",

        // Restore all registers (except rax)
        "pop rbp",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rbx",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",

        // Return from interrupt
        "iretq",

        NUM_SYSCALLS = const SyscallFunction::NumSyscalls as usize,
        SYSCALL_TABLE = sym SYSCALL_TABLE
    )
}

/// This function is called if an invalid syscall number is passed.
/// It panics and halts the system.
#[unsafe(no_mangle)]
extern "C" fn syscall_abort() {
    panic!("Invalid syscall number");
}

/// fast system calls:
/// cpu jumps directly into address stored in LSTAR_MSR & loads segment registers from STAR_MSR

/// MSR addresses and configuration values needed to set up fast system calls.
mod fast_syscall_const {
    /// Table 2-2. IA-32 Architectural MSRs (Contd.) [ ntel Software Developer’s Manual Vol. 4 - 2.1]
    /// IA32_EFER: extended feature enable register (contains the SCE bit).
    pub const MSR_EFER: u32 = 0xC000_0080;
    /// IA32_EFER.SCE: enables the syscall/sysret instructions.
    pub const EFER_SCE: u64 = 1;
    /// IA32_STAR: contains the selectors for syscall (kernel) and sysret (user).
    pub const MSR_STAR: u32 = 0xC000_0081;
    /// IA32_LSTAR: entry point of the syscall handler (RIP on syscall).
    pub const MSR_LSTAR: u32 = 0xC000_0082;
    /// IA32_FMASK: RFLAGS bits that are cleared during a syscall.
    pub const MSR_FMASK: u32 = 0xC000_0084;

    /// Selector of the kernel code segment in the GDT (written to STAR[32:47]).
    /// On syscall, CS is loaded with this selector and SS with selector + 8 (kernel data).
    pub const KERNEL_CODE_SELECTOR: u64 = 4 * 8;
    /// Base selector for the user code/data segments (written to STAR[48:63]).
    /// On sysret, SS is loaded with selector + 8 (user data) and CS with selector + 16 (user code).
    pub const USER_SEGMENTS_BASE_SELECTOR: u64 = 1 * 8;

    /// RFLAGS bits to clear during a syscall: IOPL, NT, RF, VM, VIF, VIP.
    /// The interrupts flag is not masked, so the syscall handler runs with interrupts enabled
    /// (the same behavior as the interrupt-based syscall via trap gate).
    pub const FMASK: u64 = (0b11 << 12) | (1 << 14) | (1 << 16) | (1 << 17) | (1 << 19) | (1 << 20);
}

pub fn init_fast_syscalls() {
    use fast_syscall_const::*;

    // Enable syscall/sysret in EFER (read-modify-write, since LMA/LME are already set)
    let efer = crate::device::cpu::read_msr(MSR_EFER);
    crate::device::cpu::write_msr(MSR_EFER, efer | EFER_SCE);

    // Load selectors for kernel/user mode into the STAR register
    let star = (USER_SEGMENTS_BASE_SELECTOR << 48) | (KERNEL_CODE_SELECTOR << 32);
    crate::device::cpu::write_msr(MSR_STAR, star);

    // Load FMASK and the entry point
    crate::device::cpu::write_msr(MSR_FMASK, FMASK);
    crate::device::cpu::write_msr(MSR_LSTAR, _fast_dispatch_syscall as *const () as u64);
}

/// Fast system call dispatcher.
/// This function is called on the 'syscall' instruction (its address is stored in the LSTAR-MSR).
/// The syscall number is passed in the rax register and is used as an index into
/// the syscall table to call the corresponding function.
/// All registers (except rax) are saved and restored.
/// The syscall function's return value is passed back in rax.
///
/// Since 'syscall' does not switch stacks (unlike 'int'), everything happens on the *user* stack.
/// On syscall, RCX contains the return address in user mode and R11 contains the user RFLAGS.
#[unsafe(naked)]
pub extern "C" fn _fast_dispatch_syscall() {
    naked_asm!(
        // rcx = user return address, r11 = user RFLAGS
        "mov r10, rsp", // save user stack pointer (the handler runs on the user stack!)
        "and rsp, -16", // align the stack for the SysV ABI call below

        // Save the user stack pointer (restored with 'pop rsp' before sysret)
        "push r10",

        // Save registers that are used implicitly by sysret
        "push rcx", // user return address
        "push r11", // user RFLAGS

        // Save all remaining registers (except rax, which contains the syscall number)
        "push r8",
        "push r9",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "push rbx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push rbp",

        // The fast user stubs pass the fourth parameter in r9, since rcx is used by 'syscall'.
        // Move it to rcx, where the SysV ABI expects the fourth parameter.
        "mov rcx, r9",

        // Call syscall handler (or syscall_abort for an invalid syscall number)
        "cmp rax, {NUM_SYSCALLS}",
        "jae syscall_abort",
        "call [{SYSCALL_TABLE} + rax * 8]",

        // Restore all registers (except rax)
        "pop rbp",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rbx",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r9",
        "pop r8",

        // Restore rcx/r11 and the user stack pointer, then return to user mode
        "pop r11",
        "pop rcx",
        "pop rsp",
        "sysretq",

        NUM_SYSCALLS = const SyscallFunction::NumSyscalls as usize,
        SYSCALL_TABLE = sym SYSCALL_TABLE
    )
}
