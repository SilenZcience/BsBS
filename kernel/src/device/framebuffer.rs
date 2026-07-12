/*
 * Driver for a linear framebuffer in 32-bit RGB format.
 *
 * Author: Michael Schoetter, Heinrich Heine University Duesseldorf, 2023-06-26
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-07
 * License: GPLv3
 */
use core::cmp::max;
use crate::library::bitmap::Bitmap;
use crate::multiboot;

/// Represents a linear framebuffer for graphics output.
/// The framebuffer is expected to be in 32-bit RGB format.
pub struct Framebuffer {
    /// The width of the framebuffer in pixels.
    pub width: usize,
    /// The height of the framebuffer in pixels.
    pub height: usize,
    /// The number of bytes per row of pixels.
    /// This may be greater than (width * 4) due to padding.
    pitch: usize,
    /// A pointer to the start of the framebuffer memory.
    address: u64,
}

/// Create a 32-bit color value from red, green, and blue components.
/// Each component is an 8-bit value (0-255).
/// The resulting color is in the format 0x00RRGGBB.
pub const fn color(red: u8, green: u8, blue: u8) -> u32 {
    ((red as u32) << 16) | ((green as u32) << 8) | (blue as u32)
}

// ANSI colors
pub const BLACK: u32 = color(0, 0, 0);
pub const RED: u32 = color(170, 0, 0);
pub const GREEN: u32 = color(0, 170, 0);
pub const YELLOW: u32 = color(170, 170, 0);
pub const BROWN: u32 = color(170, 85, 0);
pub const BLUE: u32 = color(0, 0, 170);
pub const MAGENTA: u32 = color(170, 0, 170);
pub const CYAN: u32 = color(0, 170, 170);
pub const WHITE: u32 = color(170, 170, 170);

pub const CHAR_WIDTH: usize = 8;
pub const CHAR_HEIGHT: usize = 16;

impl Framebuffer {
    /// Create a new Framebuffer instance.
    /// This function is unsafe because the caller must ensure that the provided
    /// buffer pointer is valid and points to a memory region large enough to hold
    /// the framebuffer data.
    pub const unsafe fn new(width: usize, height: usize, pitch: usize, address: u64) -> Framebuffer {
        Framebuffer { width, height, pitch, address }
    }

    /// Create a Framebuffer from multiboot framebuffer information.
    /// Returns None if the framebuffer type is not supported or if the bits per pixel is not 32.
    /// This function is safe, because it assumes the multiboot information is valid.
    pub const fn from_multiboot(info: &multiboot::FramebufferInfo) -> Option<Framebuffer> {
        match info.typ {
            multiboot::FramebufferType::RGB => {
                if info.bpp != 32 {
                    None
                } else {
                    Some(Framebuffer {
                        width: info.width as usize,
                        height: info.height as usize,
                        pitch: info.pitch as usize,
                        address: info.address,
                    })
                }
            },
            _ => None,
        }
    }

    /// Get the width of the framebuffer in pixels.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the height of the framebuffer in pixels.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Clear the framebuffer by filling it with black pixels.
    pub fn clear(&mut self) {
        let buffer = self.address as *mut u8;
        unsafe { buffer.write_bytes(0, self.pitch * self.height); }
    }

    /// Draw a pixel at the specified (x, y) coordinates with the given color.
    /// This method checks the bounds of the framebuffer before drawing
    /// and omits drawing if the coordinates are out of bounds.
    pub fn draw_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            unsafe { self.draw_pixel_unchecked(x, y, color); }
        }
    }

    /// Draw a pixel at the specified (x, y) coordinates with the given color.
    /// This method does not check the bounds of the framebuffer.
    /// This is faster than `draw_pixel` but the caller must ensure that the coordinates are valid.
    /// Drawing outside the framebuffer may lead to undefined behavior.
    pub unsafe fn draw_pixel_unchecked(&mut self, x: usize, y: usize, color: u32) {
        let offset = y * self.pitch + x * 4;

        let buffer = self.address as *mut u8;
        unsafe { buffer.add(offset).cast::<u32>().write_volatile(color); }
    }

    /// Get the pixel data for a character from the font data.
    // fn get_char_pixels(c: char) -> &'static [u8] {
    //     let char_mem_size = (font_8x8::CHAR_WIDTH + (8 >> 1)) / 8 * font_8x8::CHAR_HEIGHT;
    //     let start = char_mem_size * c as usize;
    //     let end = start + char_mem_size;

    //     &font_8x8::DATA[start..end]
    // }

    /// Draw a single character at the specified (x, y) coordinates with the given foreground and background colors.
    /// If the character does not fit fully within the framebuffer, it is not drawn.
    pub fn draw_char(&mut self, c: char, x: usize, y: usize, fg_color: u32, bg_color: u32) {
        if x + CHAR_WIDTH > self.width || y + CHAR_HEIGHT > self.height {
            return;
        }

        if let Some(glyph) = unifont::get_glyph(c) {
            if glyph.get_width() != CHAR_WIDTH {
                return;
            }

            for y_offset in 0..CHAR_HEIGHT {
                for x_offset in 0..CHAR_WIDTH {
                    if glyph.get_pixel(x_offset, y_offset) {
                        unsafe { self.draw_pixel_unchecked(x + x_offset, y + y_offset, fg_color); }
                    } else {
                        unsafe { self.draw_pixel_unchecked(x + x_offset, y + y_offset, bg_color); }
                    }
                }
            }
        } else {
            for y_offset in 0..CHAR_HEIGHT {
                for x_offset in 0..CHAR_WIDTH {
                    unsafe { self.draw_pixel_unchecked(x + x_offset, y + y_offset, bg_color); }
                }
            }
        }
    }

    /// Draw a string at the specified (x, y) coordinates with the given foreground and background colors.
    pub fn draw_str(&mut self, str: &str, x: usize, y: usize, fg_color: u32, bg_color: u32) {
        let mut x = x;

        for c in str.chars() {
            self.draw_char(c, x, y, fg_color, bg_color);
            x += CHAR_WIDTH;
        }
    }

    /// Scroll the framebuffer content up by the specified number of lines.
    /// The freed space at the bottom is cleared to black.
    pub fn scroll_up(&mut self, lines: usize) {
        if lines == 0 || lines >= self.height / CHAR_HEIGHT {
            self.clear();
            return;
        }

        let char_height = CHAR_HEIGHT;
        let scroll_px = lines * char_height;
        let total_bytes = self.pitch * self.height;
        let move_bytes = (self.height - scroll_px) * self.pitch;

        let buffer = self.address as *mut u8;
        unsafe {
            // Move framebuffer content up (correct offset in bytes)
            core::ptr::copy(
                buffer.add(scroll_px * self.pitch),
                buffer,
                move_bytes,
            );
            // Clear the bottom area
            buffer.add(move_bytes).write_bytes(0, total_bytes - move_bytes);
        }
    }

    /// Draw a bitmap image at the specified (x, y) coordinates.
    /// If the bitmap does not fully fit within the framebuffer, it is clipped.
    pub fn draw_bitmap(&mut self, bitmap: &Bitmap, x: usize, y: usize) {
        // Original bitmap dimensions
        let bmp_width = bitmap.width() as usize;
        let bmp_height = bitmap.height() as usize;

        // Clip the bitmap to the framebuffer dimensions
        let target_width = if x + bmp_width > self.width {
            max(self.width - x, 0)
        } else {
            bmp_width
        };

        let target_height = if y + bmp_height > self.height {
            max(self.height - y, 0)
        } else {
            bmp_height
        };

        todo!("framebuffer::draw_bitmap() is not yet implemented");
    }
}
