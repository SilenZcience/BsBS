/*
 * Frontend for the Peanut-GB emulator.
 * ROMs are loaded from the filesystem, and the Game Boy screen is rendered to the framebuffer.
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-04-01
 * License: GPLv3
 */

use alloc::vec::Vec;
use core::ffi::{c_char, c_int, c_size_t, c_void, CStr};
use core::fmt::Write;
use log::{error, info};
use crate::library::once::Once;
use crate::library::spinlock::Spinlock;

/// Debug info struct matching the C struct `gb_cart_debug_info`.
#[repr(C)]
struct GbCartDebugInfo {
    enable_cart_ram: u8,
    cart_ram: u8,
    mbc: u8,
    cart_ram_bank: u8,
    cart_mode_select: u8,
    num_ram_banks: u32,
    num_rom_banks_mask: u32,
    selected_rom_bank: u32,
}

unsafe extern "C" {
    /// Get the size of the `gb_s` structure (implemented in `peanut-gb.c`).
    /// This struct holds the entire state of the emulated Game Boy.
    /// Since we do not have a Rust binding for this, we use a C function to get the size.
    fn gb_size() -> c_int;

    /// Get a pointer to the joypad state in the `gb_s` structure (implemented in `peanut-gb.c`).
    /// The joypad state is a single byte where each bit represents a button state.
    /// If no button is pressed, all bits are set to 1 (0xff).
    /// The buttons are represented by the `JoypadButton` enum.
    fn gb_get_joypad_ptr(gb: *mut c_void) -> *mut u8;

    /// Get debug info about the cart RAM state from the `gb_s` structure.
    fn gb_debug_cart_info(gb: *mut c_void, info: *mut GbCartDebugInfo);

    /// Initialization function for the PeanutGB emulator.
    /// The `gb` parameter must point to block of memory large enough to hold the `gb_s` structure.
    /// The size of this structure can be obtained by calling `gb_size()`.
    /// The `priv_data` parameter can be used to pass additional data to the emulator,
    /// but is currently unused in this implementation.
    /// The other parameters are function pointers and crucial for the emulator to function.
    fn gb_init(gb: *mut c_void,
               gb_rom_read: unsafe extern "C" fn(*mut c_void, u32) -> u8,
               gb_cart_ram_read: unsafe extern "C" fn(*mut c_void, u32) -> u8,
               gb_cart_ram_write: unsafe extern "C" fn(*mut c_void, u32, u8),
               gb_error: unsafe extern "C" fn(*mut c_void, i32, u16),
               priv_data: *const c_void) -> c_int;

    /// Initialize the LCD of the PeanutGB emulator.
    /// This function must be called after the emulator has been initialized.
    /// If this function is not called, the emulator will work, but not render any graphics.
    fn gb_init_lcd(gb: *mut c_void, lcd_draw_line: *const c_void);

    /// Run a single frame of the PeanutGB emulator.
    /// This function must be called in a loop to run the emulator.
    /// To maintain a stable frame rate, the caller should measure the time taken by this function
    /// and sleep for the remaining time to achieve the desired frame rate.
    /// Otherwise, the emulator will run as fast as possible.
    fn gb_run_frame(gb: *mut c_void);

    /// Get the name of the ROM currently loaded in the PeanutGB emulator.
    /// The name is returned as a C string (null-terminated).
    fn gb_get_rom_name(gb: *mut c_void, title_str: *const c_char) -> *const c_char;

    /// Get the RAM size of the currently loaded ROM in the PeanutGB emulator.
    /// The RAM size is written to the given pointer `ram_size`.
    /// A return value of 0 indicates success.
    fn gb_get_save_size_s(gb: *mut c_void, ram_size: *mut c_size_t) -> c_int;
}

/// Bitmask for the joypad buttons. See `gb_get_joypad_ptr` for more details.
#[repr(u8)]
enum JoypadButton {
    A = 0x01,
    B = 0x02,
    Select = 0x04,
    Start = 0x08,
    Right = 0x10,
    Left = 0x20,
    Up = 0x40,
    Down = 0x80,
}

/// Error codes used in `gb_error`.
#[derive(Debug, PartialEq)]
enum GbError {
    UnknownError = 0,
    InvalidOpcode = 1,
    InvalidRead = 2,
    InvalidWrite = 3,
}

impl TryFrom<c_int> for GbError {
    type Error = ();

    fn try_from(value: c_int) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(GbError::UnknownError),
            1 => Ok(GbError::InvalidOpcode),
            2 => Ok(GbError::InvalidRead),
            3 => Ok(GbError::InvalidWrite),
            _ => Err(())
        }
    }
}

/// Error codes used in `gb_init`.
#[derive(Debug, PartialEq)]
enum GbInitError {
    NoError = 0,
    CartridgeUnsupported,
    InvalidChecksum,
    UnknownError = 0xff
}

impl TryFrom<c_int> for GbInitError {
    type Error = ();

    fn try_from(value: c_int) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(GbInitError::NoError),
            1 => Ok(GbInitError::CartridgeUnsupported),
            2 => Ok(GbInitError::InvalidChecksum),
            3 => Ok(GbInitError::UnknownError),
            _ => Err(())
        }
    }
}

/// The target frame rate for the emulator.
/// The original Game Boy runs at 60 frames per second.
/// Increasing this value will make the emulator run faster,
/// decreasing it will make the emulator run slower.
const TARGET_FRAME_RATE: usize = 60;

/// The number of milliseconds per frame at the target frame rate.
const MS_PER_FRAME: usize = 1000 / TARGET_FRAME_RATE;

/// The original Game Boy screen resolution (160x144 pixels).
const GB_SCREEN_RES: (usize, usize) = (160, 144);

const SCALE: usize = 2;

/// The color palette used for rendering.
/// The Game Boy supports 4 shades of gray, represented as 32-bit ARGB colors in this array.
static PALETTE: &[u32] = &[
    0xe0f8d0, // White
    0x88c070, // Light Gray
    0x346856, // Dark Gray
    0x081820, // Black
];

/// The ROM file to be played by the emulator.
static ROM: Once<Vec<u8>> = Once::new();

/// The battery-backed cartridge RAM.
static CART_RAM: Spinlock<Vec<u8>> = Spinlock::new(alloc::vec::Vec::new());

/// Counter for cart RAM writes (for debugging).
static CART_RAM_WRITE_COUNT: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

/// Counter for cart RAM reads (for debugging).
static CART_RAM_READ_COUNT: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

/// Read a byte from the ROM file at the offset specified by `addr`.
/// This is a callback function for the PeanutGB emulator.
unsafe extern "C" fn gb_rom_read(_gb: *mut c_void, addr: u32) -> u8 {
    ROM.get().unwrap()[addr as usize]
}

/// Read a byte from the save RAM at the offset specified by `addr`.
/// This is a callback function for the PeanutGB emulator.
///
/// This is mostly needed for save game support and part of an optional assignment.
unsafe extern "C" fn gb_cart_ram_read(_gb: *mut c_void, addr: u32) -> u8 {
    CART_RAM_READ_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    CART_RAM.lock()[addr as usize]
}

/// Write a byte to the save RAM at the offset specified by `addr`.
/// This is a callback function for the PeanutGB emulator.
///
/// This is mostly needed for save game support and part of an optional assignment.
unsafe extern "C" fn gb_cart_ram_write(_gb: *mut c_void, addr: u32, val: u8) {
    CART_RAM_WRITE_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    CART_RAM.lock()[addr as usize] = val;
}

/// Draw a line of pixels from the Game Boy screen to the framebuffer.
/// The buffer pointed to by `pixels` contains the pixel data for the line.
/// Each pixel is represented by a single byte, whose first two bits represent the color index.
/// The other bits are used for Game Boy Color emulation, but are ignored in this implementation.
unsafe extern "C" fn lcd_draw_line(_gb: *mut c_void, pixels: *const u8, line: u8) {
    let mut fb = crate::device::terminal::framebuffer().lock();
    let line = line as usize;
    let x_offset = (fb.width - GB_SCREEN_RES.0 * SCALE) / 2;
    let y_offset = (fb.height - GB_SCREEN_RES.1 * SCALE) / 2;
    for x in 0..GB_SCREEN_RES.0 {
        let pixel = unsafe { *pixels.add(x) };
        let color = PALETTE[(pixel & 0x03) as usize];
        for sy in 0..SCALE {
            for sx in 0..SCALE {
                fb.draw_pixel(x * SCALE + sx + x_offset, line * SCALE + sy + y_offset, color);
            }
        }
    }
}

/// Handle emulation errors.
/// This is a callback function for the PeanutGB emulator.
unsafe extern "C" fn gb_error(_gb: *mut c_void, error: c_int, addr: u16) {
    let error = GbError::try_from(error).unwrap_or(GbError::UnknownError);
    error!("PeanutGB error [{:?}] at address [0x{:0>4x}]!", error, addr);
}

/// Play the given ROM file using the Peanut-GB emulator.
pub fn play(rom_path: &str) {
    use crate::device::key::Scancode;
    use crate::device::pit;
    use crate::device::keyboard::keyboard_buffer;
    use crate::device::serial::COM3;

    let fs = crate::filesystem::tarfs::filesystem();
    let handle = fs.open(rom_path).expect("Failed to open ROM file");
    let size = fs.size(handle).expect("Failed to get ROM size");
    let mut rom = alloc::vec![0u8; size];
    fs.read(handle, &mut rom).expect("Failed to read ROM file");
    let _ = fs.close(handle);
    ROM.init(|| rom);

    let gb_size = unsafe { gb_size() } as usize;
    let mut gb_struct = Vec::<u8>::with_capacity(gb_size);
    let gb_ptr = gb_struct.as_mut_ptr() as *mut c_void;

    let result = unsafe {
        gb_init(
            gb_ptr,
            gb_rom_read,
            gb_cart_ram_read,
            gb_cart_ram_write,
            gb_error,
            core::ptr::null(),
        )
    };
    let result = GbInitError::try_from(result).unwrap_or(GbInitError::UnknownError);
    if result != GbInitError::NoError {
        panic!("Failed to initialize PeanutGB (Error: {:?})", result);
    }

    // {
    //     let rom = ROM.get().unwrap();
    //     if rom.len() > 0x0149 {
    //         let mbc_type = rom[0x0147];
    //         let ram_size_code = rom[0x0149];
    //         info!("ROM header: MBC type=0x{:02X}, RAM size code=0x{:02X}", mbc_type, ram_size_code);
    //     }
    // }

    let mut ram_size: c_size_t = 0;
    let ram_result = unsafe { gb_get_save_size_s(gb_ptr, &mut ram_size) };
    // info!("gb_get_save_size_s returned ram_result={}, ram_size={}", ram_result, ram_size);
    if ram_result == 0 && ram_size > 0 {
        *CART_RAM.lock() = alloc::vec![0u8; ram_size];

        // Build save file path from ROM path: "roms/2048.gb" -> "roms/2048.sav"
        let save_file_path = {
            let path_str = rom_path;
            if let Some(dot_pos) = path_str.rfind('.') {
                let base = &path_str[..dot_pos];
                let mut result = alloc::string::String::new();
                result.push_str(base);
                result.push_str(".sav");
                result
            } else {
                let mut result = alloc::string::String::new();
                result.push_str(path_str);
                result.push_str(".sav");
                result
            }
        };

        if let Ok(handle) = fs.open(&save_file_path) {
            if let Ok(file_size) = fs.size(handle) {
                let copy_len = file_size.min(ram_size);
                let mut buf = alloc::vec![0u8; copy_len];
                if let Ok(n) = fs.read(handle, &mut buf) {
                    let mut ram = CART_RAM.lock();
                    ram[..n].copy_from_slice(&buf[..n]);
                    info!("Loaded save file '{}' ({} bytes)", save_file_path, n);
                    // if n >= 64 {
                    //     info!("CART_RAM[0x0000..0x0080]: {:02X?}", &ram[..0x80]);
                    //     info!("CART_RAM[0x0100..0x0180]: {:02X?}", &ram[0x100..0x180]);
                    //     info!("CART_RAM[0x1000..0x1080]: {:02X?}", &ram[0x1000..0x1080]);
                    //     info!("CART_RAM[0x1F00..0x1F80]: {:02X?}", &ram[0x1F00..0x1F80]);
                    //     info!("CART_RAM[0x2000..0x2080]: {:02X?}", &ram[0x2000..0x2080]);
                    // }
                }
            }
            let _ = fs.close(handle);
        } else {
            info!("No save file found at '{}'", save_file_path);
        }
    }

    unsafe { gb_init_lcd(gb_ptr, lcd_draw_line as *const c_void); }

    let joypad_ptr = unsafe { gb_get_joypad_ptr(gb_ptr) };
    let mut running = true;
    let mut frame_count: usize = 0;
    let mut last_fps_time = pit::system_time();

    while running {
        let frame_start = pit::system_time();

        while let Some(event) = keyboard_buffer().pop_key_event() {
            if let Some(scancode) = event.scancode() {
                if scancode == Scancode::Escape && event.pressed() {
                    running = false;
                    break;
                }
                let pressed = event.pressed();
                let bit = match scancode {
                    Scancode::W => Some(JoypadButton::Up as u8),
                    Scancode::S => Some(JoypadButton::Down as u8),
                    Scancode::A => Some(JoypadButton::Left as u8),
                    Scancode::D => Some(JoypadButton::Right as u8),
                    Scancode::Q => Some(JoypadButton::A as u8),
                    Scancode::E => Some(JoypadButton::B as u8),
                    Scancode::Space => Some(JoypadButton::Start as u8),
                    Scancode::Enter => Some(JoypadButton::Select as u8),
                    _ => None,
                };
                if let Some(mask) = bit {
                    unsafe {
                        if pressed {
                            *joypad_ptr &= !mask;
                        } else {
                            *joypad_ptr |= mask;
                        }
                    }
                }
            }
        }

        if !running {
            break;
        }

        unsafe { gb_run_frame(gb_ptr); }

        frame_count += 1;
        let now = pit::system_time();
        let fps_elapsed = now - last_fps_time;
        if fps_elapsed >= 1000 {
            let fps = frame_count * 1000 / fps_elapsed;
            frame_count = 0;
            last_fps_time = now;

            // let reads = CART_RAM_READ_COUNT.swap(0, core::sync::atomic::Ordering::Relaxed);
            // let writes = CART_RAM_WRITE_COUNT.swap(0, core::sync::atomic::Ordering::Relaxed);
            // info!("cart_ram_reads:{} cart_ram_writes:{}", reads, writes);

            // let mut dbg = GbCartDebugInfo {
            //     enable_cart_ram: 0, cart_ram: 0, mbc: 0,
            //     cart_ram_bank: 0, cart_mode_select: 0,
            //     num_ram_banks: 0, num_rom_banks_mask: 0, selected_rom_bank: 0,
            // };
            // unsafe { gb_debug_cart_info(gb_ptr, &mut dbg); }
            // info!("MBC state: mbc={} enable_cart_ram={} cart_ram={} ram_banks={} cart_ram_bank={} mode_sel={} rom_bank={}/{}",
            //     dbg.mbc, dbg.enable_cart_ram, dbg.cart_ram, dbg.num_ram_banks,
            //     dbg.cart_ram_bank, dbg.cart_mode_select, dbg.selected_rom_bank, dbg.num_rom_banks_mask + 1);

            let mut fps_str = alloc::string::String::new();
            let _ = write!(&mut fps_str, "FPS:{}", fps);
            while fps_str.len() < 8 {
                fps_str.push(' ');
            }

            let mut fb = crate::device::terminal::framebuffer().lock();
            let str_pixel_width = fps_str.len() * crate::device::framebuffer::CHAR_WIDTH;
            let x = fb.width.saturating_sub(str_pixel_width);
            let y = fb.height.saturating_sub(crate::device::framebuffer::CHAR_HEIGHT);
            fb.draw_str(&fps_str, x, y, crate::device::framebuffer::WHITE, crate::device::framebuffer::BLACK);
        }

        let elapsed = pit::system_time() - frame_start;
        if elapsed < MS_PER_FRAME {
            pit::wait(MS_PER_FRAME - elapsed);
        }
    }

    // {
    //     let ram = CART_RAM.lock();
    //     info!("Export: CART_RAM[0x0000..0x0080]: {:02X?}", &ram[..0x80]);
    //     info!("Export: CART_RAM[0x0100..0x0180]: {:02X?}", &ram[0x100..0x180]);
    //     info!("Export: CART_RAM[0x1000..0x1080]: {:02X?}", &ram[0x1000..0x1080]);
    //     info!("Export: CART_RAM[0x1F00..0x1F80]: {:02X?}", &ram[0x1F00..0x1F80]);
    //     info!("Export: CART_RAM[0x2000..0x2080]: {:02X?}", &ram[0x2000..0x2080]);
    // }
    let ram = CART_RAM.lock();
    let mut com3 = COM3.lock();
    com3.init();
    info!("Writing {} cart RAM bytes to COM3 for save", ram.len());
    if ram.is_empty() {
        info!("CART_RAM is empty - nothing to save!");
    }
    for &byte in ram.iter() {
        com3.write_raw_byte(byte);
    }
    info!("Save data sent to COM3 serial");
}
