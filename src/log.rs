//! Everything goes to stderr: stdout is reserved for output other programs consume, like the path
//! `switch` prints for the shell function to cd into.

const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const NC: &str = "\x1b[0m";

pub fn info(msg: &str) {
    eprintln!("{GREEN}{msg}{NC}");
}

pub fn error(msg: &str) {
    eprintln!("{RED}{msg}{NC}");
}
