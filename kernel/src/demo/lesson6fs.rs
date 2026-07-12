

use log::info;
use crate::device::terminal;


pub fn text_file_demo() {
    let fs = crate::filesystem::tarfs::filesystem();
    let filename = "lorem.txt";
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

pub fn bitmap_demo() {
    let bmp_filename = "heine.bmp";
    match crate::library::bitmap::Bitmap::read_from_file(bmp_filename) {
        Ok(Some(bitmap)) => {
            info!("Loaded bitmap: {}x{}", bitmap.width(), bitmap.height());
            let mut fb = terminal::framebuffer().lock();
            let x = (fb.width() - bitmap.width() as usize) / 2;
            let y = (fb.height() - bitmap.height() as usize) / 2;
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
