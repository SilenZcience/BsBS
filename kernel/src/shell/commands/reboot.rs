use alloc::string::String;
use crate::shell::registry;

pub fn register() {
    registry::register("reboot", "Reboot the system", run);
}

fn run(_args: &[String]) {
    uefi::runtime::reset(uefi::runtime::ResetType::COLD, uefi::Status::SUCCESS, None);
}
