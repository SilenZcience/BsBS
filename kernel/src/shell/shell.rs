use crate::device::terminal::PROMPT_COLOR;
use crate::shell::parser::parse_line;
use crate::shell::readline::read_line;
use crate::shell::commands;
use crate::shell::aliases;
use crate::shell::registry;
use alloc::vec::Vec;
use alloc::string::String;

pub fn print_prompt() {
    print_colored!("kernel@HeineOS:-$ ", PROMPT_COLOR);
}

pub fn run_shell() -> ! {
    commands::register_all();
    aliases::load_aliases();

    let mut all_commands_names: Vec<String> = registry::list()
        .iter()
        .map(|cmd| String::from(cmd.name))
        .collect();
    let aliases = aliases::list();
    let aliases_names: Vec<String> = aliases.into_iter().map(|(alias_name, _)| alias_name).collect();
    all_commands_names.extend(aliases_names);

    println!("HeineOS Shell - Type 'help' for commands");
    loop {
        print_prompt();
        let line = read_line(&mut all_commands_names);
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let parsed = match parse_line(line) {
            Some(parsed) => parsed,
            None => continue,
        };

        let expanded_cmd = aliases::expand_aliases(parsed.command());
        let mut expanded_parts = expanded_cmd.split_whitespace();
        let cmd_name = expanded_parts.next().unwrap_or(parsed.command());
        let mut args: Vec<String> = expanded_parts.map(String::from).collect();
        args.extend(parsed.args().iter().cloned());

        match registry::find(cmd_name) {
            Some(cmd) => (cmd.run)(&args),
            None => {
                println!("Unknown command: '{}'", parsed.command());
                println!("Type 'help' for available commands");
            }
        }
    }
}
