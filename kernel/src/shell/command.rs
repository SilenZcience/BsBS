use crate::device::terminal::PROMPT_COLOR;
use crate::shell::commands;
use crate::shell::aliases;
use crate::shell::parser::parse_line;
use crate::shell::readline::read_line;
use crate::shell::registry;


pub fn run_shell() -> ! {
    commands::register_all();
    aliases::load_aliases();

    println!("HeineOS Shell - Type 'help' for commands");
    loop {
        print_colored!("User@HeineOS:-$ ", PROMPT_COLOR);
        let line = read_line();
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let parsed = match parse_line(line) {
            Some(parsed) => parsed,
            None => continue,
        };

        match registry::find(&aliases::expand_aliases(parsed.command())) {
            Some(cmd) => (cmd.run)(parsed.args()),
            None => {
                println!("Unknown command: '{}'", parsed.command());
                println!("Type 'help' for available commands");
            }
        }
    }
}
