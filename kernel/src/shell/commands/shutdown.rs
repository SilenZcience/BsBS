use alloc::string::String;
use crate::shell::registry;

pub fn register() {
    registry::register("shutdown", "Power off the system", run);
}

fn run(_args: &[String]) {
    uefi::runtime::reset(uefi::runtime::ResetType::SHUTDOWN, uefi::Status::SUCCESS, None);
}
