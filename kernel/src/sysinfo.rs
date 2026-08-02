use alloc::string::String;
use crate::library::once::Once;

static BOOTLOADER_NAME: Once<String> = Once::new();

pub fn set_bootloader_name(name: &str) {
    BOOTLOADER_NAME.init(|| String::from(name));
}

pub fn bootloader_name() -> Option<&'static str> {
    BOOTLOADER_NAME.get().map(|name| name.as_str())
}
