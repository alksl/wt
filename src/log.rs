//! Everything goes to stderr: stdout is reserved for output other programs consume, like the path
//! `switch` prints for the shell function to cd into. Colour is dropped when stderr isn't a
//! terminal, and follows `NO_COLOR` / `CLICOLOR_FORCE`.

use anstyle::{AnsiColor, Style};

const RED: Style = AnsiColor::Red.on_default();
const GREEN: Style = AnsiColor::Green.on_default();
const YELLOW: Style = AnsiColor::Yellow.on_default();

pub fn info(msg: &str) {
    print(GREEN, msg);
}

pub fn warn(msg: &str) {
    print(YELLOW, msg);
}

pub fn error(msg: &str) {
    print(RED, msg);
}

fn print(style: Style, msg: &str) {
    anstream::eprintln!("{style}{msg}{style:#}");
}
