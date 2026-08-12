use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;

pub struct ParsedLine {
    command: String,
    args: Vec<String>,
}

impl ParsedLine {
    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }
}

pub fn parse_line(line: &str) -> Option<ParsedLine> {
    let mut parts = line.split_whitespace();
    let command = parts.next()?.to_owned();

    Some(ParsedLine {
        command,
        args: parts.map(str::to_owned).collect(),
    })
}
