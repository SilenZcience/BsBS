/*
 * Contains functions to create, start, switch and end threads.
 *
 * Author: Michael Schoettner, Heinrich Heine University Duesseldorf, 2023-05-15
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-15
 * License: GPLv3
 */

use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::vec;
use core::{fmt, ptr};
use core::arch::naked_asm;
use core::fmt::Display;
use core::sync::atomic::AtomicUsize;
use crate::consts::STACK_SIZE;
use crate::device::cpu;
use crate::thread::scheduler::scheduler;

static THREAD_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn next_id() -> usize {
    THREAD_ID_COUNTER.fetch_add(1, core::sync::atomic::Ordering::SeqCst)
}

// Low-level routine for storing `rsp0` in the current TSS
unsafe extern "C" {
    fn tss_set_rsp0(rsp0: usize);
}

/// Low-level routine for starting a thread.
#[unsafe(naked)]
unsafe extern "C" fn thread_start(stack_ptr: usize) {
    naked_asm!(
        "mov rsp, rdi",
        "call unlock_scheduler",
        "popfq",
        "pop rbp",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rbx",
        "pop rax",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "ret"
    )
}

/// Low-level routine for switching to the next thread.
/// `current_stack_ptr` is a pointer to `stack_ptr` of the next coroutine (where the rsp is saved).
/// `next_stack` is the value of `stack_ptr` of the next thread (the new rsp value).
/// `next_stack_end` points to the top of the next thread's stack (used to set the rsp0 entry of the TSS)
#[unsafe(naked)]
unsafe extern "C" fn thread_switch(current_stack_ptr: *mut usize, next_stack: usize, next_stack_end: usize) {
    naked_asm!(
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "push rax",
        "push rbx",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push rbp",
        "pushfq",                // rflags
        "mov [rdi], rsp",        // save current stack pointer
        "mov rdi, rdx",          // As it is the third parameter, it is passed to thread_switch() in the rdx register (as specified by the System V ABI)
        "call tss_set_rsp0",     // When calling tss_set_rsp0(), the address should be passed in the rdi register
        "mov rsp, rsi",          // switch to next stack
        "call unlock_scheduler", // unlock scheduler, prevent deadlock
        "popfq",
        "pop rbp",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rbx",
        "pop rax",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "ret"
    )
}

#[unsafe(naked)]
/// Called by `switch_to_usermode()`.
/// Switches to the prepared user stack and uses `iret` to return to ring 3 code.
unsafe extern "C" fn thread_user_start(stack_ptr: usize) {
    naked_asm!(
        "mov rsp, rdi", // Switch stack
        "pop rdi", // frame[0] -> RDI, RSP now = &frame[1]
        "iretq" // Return to user mode
    )
}

/// Represents a thread in the system.
/// It contains the stack and the entry function.
/// Threads must be registered in the scheduler and are run automatically once the scheduler is started.
#[repr(C)]
pub struct Thread {
    id: usize,
    is_kernel_thread: bool,
    kernel_stack: Vec<u64>, // Memory for the kernel stack
    user_stack: Vec<u64>, // Memory for the user stack
    stack_ptr: usize, // Pointer on the stack to the saved context
    entry: fn(),
    kill_handler: Option<fn()>,
}

impl Thread {
    /// Create a new thread with the given entry function.
    pub fn new_kernel_thread(entry: fn()) -> Box<Thread> {
        // Allocate memory for the stack and initialize it to zero
        let kernel_stack = vec![0u64; STACK_SIZE / 8];
        let user_stack   = vec![0u64; STACK_SIZE / 8];

        // Set the stack pointer to the top of the stack
        let stack_ptr = ptr::from_ref(&kernel_stack[kernel_stack.capacity() - 1]) as usize;

        // Create a new thread object
        let mut thread = Box::new(
            Thread {
                id: next_id(),
                is_kernel_thread: true,
                kernel_stack: kernel_stack,
                user_stack: user_stack,
                stack_ptr,
                entry,
                kill_handler: None }
        );

        // Prepare the stack for the thread so it can be started via `thread_start()`
        thread.prepare_kernel_stack();
        thread
    }

    /// Create a new user thread with the given entry function.
    pub fn new_user_thread(entry: fn()) -> Box<Thread> {
        let mut thread = Self::new_kernel_thread(entry);
        thread.is_kernel_thread = false;
        thread
    }

    /// Switch this thread from Ring 0 to Ring 3.
    /// For this, the kernel stack is prepared in a way that an 'iretq' instruction
    /// switches to user mode (Ring 3) and the user stack is used. If this function works correctly,
    /// the thread continues in user mode in the function 'kickoff_user_thread'.
    fn switch_to_usermode(&mut self) {
        // segment offset + RPL
        const USER_CODE_SELECTOR: u64 = 3 * 8 + 3; // 0x1b
        const USER_DATA_SELECTOR: u64 = 4 * 8 + 3; // 0x23

        // enable interrupts
        const USER_RFLAGS: u64 = 0x202;

        let top_of_kernel_stack = Self::get_top_of_stack(&self.kernel_stack) as usize;
        let frame = (top_of_kernel_stack - 1024) as *mut u64;

        // RSP % 16 == 8 alignement
        let top_of_user_stack = Self::get_top_of_stack(&self.user_stack) as usize;
        let user_stack_ptr = (top_of_user_stack & !0xf) - 8;

        unsafe {
            frame.add(0).write(ptr::from_mut(self) as u64);
            // interrupt frame with no error code
            frame.add(1).write((Thread::kickoff_user_thread as *const()) as u64); // RIP
            frame.add(2).write(USER_CODE_SELECTOR);                               // CS
            frame.add(3).write(USER_RFLAGS);                                      // RFLAGS
            frame.add(4).write(user_stack_ptr as u64);                            // RSP (user stack)
            frame.add(5).write(USER_DATA_SELECTOR);                               // SS

            thread_user_start(frame as usize);
        }
    }

    /// Start the thread.
    /// This function is only once by the scheduler.
    /// The scheduler does further thread switching via `switch()`.
    pub fn start(&mut self) {
        unsafe {
            thread_start(self.stack_ptr);
        }
    }

    /// Switch from the `current` thread to the `next` thread.
    /// This function is called by the scheduler to switch between threads.
    pub unsafe fn switch(current: *mut Thread, next: *mut Thread) {
        unsafe {
            thread_switch(
                ptr::from_mut(&mut (*current).stack_ptr),
                (*next).stack_ptr,
                Self::get_top_of_stack(&(*next).kernel_stack) as usize
            );
        }
    }

    /// Get the ID of the thread.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Set a cleanup handler
    pub fn set_kill_handler(&mut self, handler: fn()) {
        self.kill_handler = Some(handler);
    }

    /// Invoke the cleanup handler
    pub fn run_kill_handler(&mut self) {
        if let Some(handler) = self.kill_handler.take() {
            handler();
        }
    }

    /// Prepare the stack of a newly created thread in a way that it can be used
    /// to return to the 'kickoff' function with the thread itself as parameter.
    /// The prepared stack is used in 'thread_start' to start the first thread.
    /// Other threads are started by 'thread_switch' with the prepared stack.
    fn prepare_kernel_stack(&mut self) {
        let kickoff = (Thread::kickoff_kernel_thread as *const()) as u64;
        let thread = ptr::from_mut(self) as u64;
        let length = self.kernel_stack.len();

        // After the 16 pops and the 'ret' in 'thread_start'/'thread_switch' (17 * 8 = 136 bytes)
        // 'kickoff' is entered with RSP % 16 == 8 as required by the SysV calling convention.
        self.kernel_stack[length -  1] = 0; // padding for 16 byte alignement !
        self.kernel_stack[length -  2] = kickoff; // Address of 'kickoff'
        self.kernel_stack[length -  3] = 0; // r8
        self.kernel_stack[length -  4] = 0; // r9
        self.kernel_stack[length -  5] = 0; // r10
        self.kernel_stack[length -  6] = 0; // r11
        self.kernel_stack[length -  7] = 0; // r12
        self.kernel_stack[length -  8] = 0; // r13
        self.kernel_stack[length -  9] = 0; // r14
        self.kernel_stack[length - 10] = 0; // r15
        self.kernel_stack[length - 11] = 0; // rax
        self.kernel_stack[length - 12] = 0; // rbx
        self.kernel_stack[length - 13] = 0; // rcx
        self.kernel_stack[length - 14] = 0; // rdx
        self.kernel_stack[length - 15] = 0; // rsi
        self.kernel_stack[length - 16] = thread; // rdi -> First parameter for 'kickoff'
        self.kernel_stack[length - 17] = 0; // rbp
        self.kernel_stack[length - 18] = 0x2; // rflags (IE = 0); interrupts disabled

        self.stack_ptr = ptr::from_ref(&self.kernel_stack[length - 18]) as usize;
    }

    /// Called indirectly by using the prepared stack in 'thread_start' and 'thread_switch'.
    fn kickoff_kernel_thread(&mut self) {
        // Set TSS rsp0 to the top of the kernel stack of this thread
        unsafe {
            let rsp0 = Self::get_top_of_stack(&self.kernel_stack);
            tss_set_rsp0(rsp0 as usize);
        }

        if self.is_kernel_thread {
            cpu::enable_int(); // Interrupts are disabled during thread start
            ((*self).entry)();
        } else {
            self.switch_to_usermode();
        }

        scheduler().exit();
    }

    /// Called indirectly by using the prepared stack in 'switch_to_usermode'.
    fn kickoff_user_thread(&self) {
        ((*self).entry)();
        loop {} // User threads may currently not exit
    }

    /// Get a pointer to the top of the given stack.
    fn get_top_of_stack(stack: &Vec<u64>) -> *const u64 {
        unsafe {
            ptr::from_ref(&stack[stack.len() - 1]).offset(1)
        }
    }
}

impl Display for Thread {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "T{}", self.id)
    }
}
