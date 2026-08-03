

use log::info;
use crate::device::terminal;


pub fn text_file_demo(filename: &str) {
    let fs = crate::filesystem::tarfs::filesystem();
    if let Ok(handle) = fs.open(filename) {
        let size = fs.size(handle).unwrap_or(0);
        info!("Opened '{}' ({} bytes)", filename, size);
        let mut buf = alloc::vec![0u8; size];
        if let Ok(n) = fs.read(handle, &mut buf) {
            if let Ok(text) = core::str::from_utf8(&buf[..n]) {
                println!("File contents ('{}'):", filename);
                println!("{}", text);
            }
        }
        let _ = fs.close(handle);
    } else {
        info!("Could not open '{}'", filename);
    }
}

pub fn bitmap_demo(bmp_filename: &str, x_pos: Option<usize>, y_pos: Option<usize>) {
    match crate::library::bitmap::Bitmap::read_from_file(bmp_filename) {
        Ok(Some(bitmap)) => {
            info!("Loaded bitmap: {}x{}", bitmap.width(), bitmap.height());
            let mut fb = terminal::framebuffer().lock();
            let x = x_pos.unwrap_or_else(|| (fb.width() - bitmap.width() as usize) / 2);
            let y = y_pos.unwrap_or_else(|| (fb.height() - bitmap.height() as usize) / 2);
            fb.draw_bitmap(&bitmap, x, y);
        }
        Ok(None) => {
            info!("'{}' is not a valid bitmap", bmp_filename);
        }
        Err(e) => {
            info!("Failed to load '{}': {:?}", bmp_filename, e);
        }
    }
}
