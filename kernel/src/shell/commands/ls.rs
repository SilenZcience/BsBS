use alloc::string::String;
use crate::library::format::format_size;
use crate::shell::registry;

pub fn register() {
    registry::register("ls", "List files in the initrd", run);
}

fn run(_args: &[String]) {
    let files = crate::filesystem::tarfs::filesystem().list_files();

    if files.is_empty() {
        println!("(initrd is empty)");
        return;
    }

    for (name, size) in files {
        println!("  {:<40} {:>10} B {:>13}", name, size, format_size(*size));
    }

    let total: usize = files.iter().map(|(_, size)| size).sum();
    println!("");
    println!("{} file(s), total {}", files.len(), format_size(total));
}
