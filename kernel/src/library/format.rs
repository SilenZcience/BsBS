use alloc::string::String;

pub fn format_size(size_bytes: usize) -> String {
    const SIZE_NAME: [&str; 9] = [" B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];

    if size_bytes == 0 {
        return String::from("0  B");
    }

    let mut i = 0;
    let mut power = 1usize;
    while power <= size_bytes / 1024 && i < SIZE_NAME.len() - 1 {
        power *= 1024;
        i += 1;
    }

    let whole = size_bytes / power;
    let hundredths = ((size_bytes % power) as u128 * 100 + (power / 2) as u128) / power as u128;

    if hundredths == 100 {
        alloc::format!("{} {}", whole + 1, SIZE_NAME[i])
    } else {
        alloc::format!("{}.{:02} {}", whole, hundredths, SIZE_NAME[i])
    }
}
