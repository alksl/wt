//! Everything goes to stderr: stdout is reserved for output other programs consume, like the path
//! `switch` prints for the shell function to cd into.

const RED: &str = "\x1b[0;31m";
const NC: &str = "\x1b[0m";

pub fn error(msg: &str) {
    eprintln!("{RED}{msg}{NC}");
}
