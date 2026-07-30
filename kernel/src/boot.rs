/*
 * Contains the entry point for the kernel, as well as all necessary module declarations.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-07
 * License: GPLv3
 */

#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(unsafe_cell_access)]
#![feature(c_size_t)]

// Silence compiler warnings.
// This is done to avoid overwhelming compiler output when building the OS at the beginning.
// As you move on with the course, the warnings for unused functions or parameters will become less relevant,
// as you will be implementing more and more of the kernel. You can delete the following lines to re-enable the warnings.
#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_variables)]

use log::{debug, error, info};
use core::str::FromStr;
use uefi::mem::memory_map::MemoryMapOwned;
use crate::device::framebuffer::Framebuffer;
use crate::device::serial::COM1;
use crate::device::terminal;
use crate::logger::Logger;

#[macro_use]
mod device;
mod coroutine;
mod interrupt;
mod library;
mod thread;
mod logger;
mod multiboot;
mod consts;
mod demo;
mod allocator;
mod filesystem;

extern crate alloc;

use crate::consts::{heap_start, HEAP_SIZE};
use crate::thread::scheduler::scheduler;

unsafe extern "C" {
    fn load_gdt();
}

/// Global logger instance. This instance is initialized at the start of the kernel.
/// After initialization, it is used by the `log` crate to log messages via macros like `info!()` and `error!()`.
static LOGGER: Logger = Logger::new();

#[unsafe(no_mangle)]
/// The kernel entry point.
/// This function is called from `boot.asm` after the bare minimum setup is done.
/// It sets up all necessary kernel components and then starts the scheduler.
pub extern "C" fn main(multiboot_magic: u32, multiboot: &multiboot::BootInfo) -> ! {
    // The first thing to do is to initialize the serial port and logger.
    // Afterward, we can use logging macros like `info!()` and `error!()` and panic messages will also be logged.
    COM1.lock().init();
    if log::set_logger(&LOGGER).is_err() {
        panic!("Failed to initialize logger");
    }

    log::set_max_level(log::LevelFilter::Debug);
    let mut log_to_terminal = false;
    if let Some(cmdline) = multiboot.find_tag::<multiboot::CommandLineTag>(multiboot::TagType::CommandLine) {
        let command_line = cmdline.as_str();
        for argument in command_line.split_whitespace() {
            if let Some(level) = argument.strip_prefix("log_level=") {
                if let Ok(parsed_level) = log::LevelFilter::from_str(level) {
                    log::set_max_level(parsed_level);
                }
            } else if let Some(enabled) = argument.strip_prefix("log_to_terminal=") {
                if let Ok(parsed_enabled) = bool::from_str(enabled) {
                    log_to_terminal = parsed_enabled;
                }
            }
        }

        info!("Command line: '{}'", command_line);
        LOGGER.enable_terminal_logging(log_to_terminal);
    }

    // Check if the bootloader passed the correct multiboot magic number.
    // If not, panic immediately as we cannot rely on the multiboot information.
    if multiboot_magic != multiboot::MULTIBOOT2_MAGIC {
        panic!("Invalid multiboot magic number: {:#x}", multiboot_magic);
    }

    // Initialize the framebuffer. Afterward, we can draw to the screen.
    // We take the framebuffer information the bootloader provided via multiboot.
    let framebuffer_info = multiboot
        .find_tag::<multiboot::FramebufferInfo>(multiboot::TagType::FramebufferInfo)
        .expect("Missing framebuffer info");

    let framebuffer = Framebuffer::from_multiboot(framebuffer_info)
        .expect("Failed to initialize framebuffer");

    // Initialize the terminal for text output.
    // The terminal takes ownership of the framebuffer, so we cannot use it directly anymore after this point.
    // If you want to experiment with the framebuffer, do it before this line or comment this line out.
    // However, the `print!()` and `println!()` macros will not work then.
    terminal::init_terminal(framebuffer);

    // Exit UEFI boot services. At this point, the UEFI boot services are still active.
    // By exiting them, the UEFI BIOS frees up resources and hands over full control to the kernel.
    // Furthermore, we get the memory map, which we need to check which memory regions are free to use.
    let _ = exit_uefi_boot_services(multiboot);

    // Load the Global Descriptor Table (code in boot.asm)
    unsafe { load_gdt(); }

    info!("Initializing heap allocator");
    crate::allocator::global::init_allocator(heap_start(), HEAP_SIZE);

    if let Some(module) = multiboot.find_tag::<multiboot::ModuleTag>(multiboot::TagType::Module) {
        info!("Found module: '{}'", module.name());
        let module_data = module.as_slice();
        let archive = tar_no_std::TarArchiveRef::new(module_data)
            .expect("Failed to parse tar archive");
        crate::filesystem::tarfs::init_filesystem(archive);
        info!("Filesystem initialized");
    } else {
        panic!("No initrd module found");
    }

    info!("Initializing scheduler");
    scheduler();

    info!("Initializing interrupt dispatcher");
    crate::interrupt::dispatcher::init_interrupt_dispatcher();

    info!("Initializing IDT");
    crate::interrupt::idt::idt().load();

    info!("Initializing PIC");
    crate::device::pic::PIC.lock().init();

    info!("Initializing keyboard");
    crate::device::keyboard::plugin();

    info!("Initializing PIT");
    crate::device::pit::plugin();

    info!("Enabling interrupts");
    crate::device::cpu::enable_int();

    info!("Boot sequence finished");


    println!("Demo Menu:");
    println!("1. Text Demo");
    println!("2. Keyboard Demo");
    println!("3. Heap Demo");
    println!("4. Speaker Demo");
    println!("5. Coroutine Demo");
    println!("6. Thread Demo");
    println!("7. Text File Demo");
    println!("8. Bitmap Demo");
    println!("9. Peanut-GB Emulator");


    use crate::device::keyboard::keyboard_buffer;
    loop {
        let event = keyboard_buffer().poll_key_press();
        if let Some(c) = event.ascii() {
            match c {
                '1' => { crate::demo::lesson1::text_demo(); break; }
                '2' => { crate::demo::lesson1::keyboard_demo(); break; }
                '3' => { crate::demo::lesson2::heap_demo(); break; }
                '4' => { crate::demo::lesson2::speaker_demo(); break; }
                '5' => { crate::demo::lesson4::coroutine_demo(); break; }
                '6' => { crate::demo::lesson4::thread_demo(); break; }
                '7' => { crate::demo::lesson6fs::text_file_demo(); break; }
                '8' => { crate::demo::lesson6fs::bitmap_demo(); break; }
                '9' => { crate::demo::lesson6::peanut_gb::play("roms/pokemon.gb"); break; }
                _ => {}
            }
        }
    }

    info!("Hello from the kernel!");
    info!("The screen resolution is {}x{}!", framebuffer_info.width as usize, framebuffer_info.height as usize);

    // Endless loop, as we cannot return from main().
    loop {}
}

/// Exit UEFI boot services.
/// When the kernel start, the UEFI boot services are still active.
/// This function exits the UEFI boot services and returns the memory map.
///
/// The memory map contains information about the system memory layout,
/// showing which regions are free, reserved, or used by hardware components.
fn exit_uefi_boot_services(multiboot: &multiboot::BootInfo) -> MemoryMapOwned {
    // Check if the bootloader terminated the EFI boot services.
    // The bootloader provides this information via a multiboot tag.
    // If this tag is not present, the boot services were terminated and we cannot proceed.
    if multiboot.find_tag::<multiboot::EfiBootServicesNotTerminatedTag>(multiboot::TagType::EfiBootServicesNotTerminated).is_none() {
        panic!("EFI boot services were terminated by the bootloader");
    }

    // Retrieve the EFI image handle from the multiboot information.
    // The map_or_else() function takes two closures:
    // The first one is called if the tag is not found, causing a panic.
    // The second one is called if the tag is found, where we set the image handle and system table.
    if let Some(image_handle_tag) = multiboot.find_tag::<multiboot::Efi64BitImageHandleTag>(multiboot::TagType::Efi64BitImageHandlePointer) {
        // The tag is found, we can log the image handle address.
        debug!("EFI image is located at: {:#x}", image_handle_tag.as_ptr() as usize);

        // We use the `uefi` crate to communicate with the UEFI firmware.
        // For that, we first need to set the EFI image handle and system table pointers in the `uefi` crate.
        unsafe {
            let image_handle = uefi::Handle::from_ptr(image_handle_tag.as_ptr()).expect("Failed to get EFI image handle");
            uefi::boot::set_image_handle(image_handle);
        }

        // Now we retrieve the EFI system table pointer from the multiboot information.
        multiboot.find_tag::<multiboot::Efi64BitSystemTableTag>(multiboot::TagType::Efi64BitSystemTablePointer)
            .map_or_else(|| {
                // If the tag is not found, panic with an error message.
                panic!("Missing EFI system table pointer tag");
            },|efi_system_table_tag| {
                // The tag is found, we can log the system table address and set it in the `uefi` crate.
                debug!("EFI system table is located at: {:#x}", efi_system_table_tag.as_ptr() as usize);
                unsafe { uefi::table::set_system_table(efi_system_table_tag.as_ptr()); }
            });
    } else {
        // If the tag is not found, panic with an error message.
        panic!("Missing EFI image handle pointer tag")
    }

    // If we reach this point, both the EFI image handle and system table have been set successfully.
    // We can now exit the UEFI boot services and retrieve the memory map.
    info!("Exiting UEFI boot services...");
    unsafe { uefi::boot::exit_boot_services(None) }
}

#[panic_handler]
/// The panic handler for the kernel.
/// It logs the panic information and enters an infinite loop, halting the system.
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("Kernel panic: {}", info);
    loop {}
}
