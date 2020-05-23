use crate::time::unix_time;
use std::io;
use std::io::{Stderr, Stdout, Write};

pub struct Logger {
    stdout: Stdout,
    stderr: Stderr,
}

impl Logger {
    /// Creates a new logger which will write logs into the given `log_file`.
    pub fn new() -> Self {
        let stdout = io::stdout();
        let stderr = io::stderr();
        Self { stdout, stderr }
    }

    /// Appends both the current unix timestamp and the string to the file in CSV format, then adds
    /// a newline.
    pub fn log<S: AsRef<str>>(&mut self, string: S) {
        let log_time = unix_time();
        write!(&mut self.stdout, "{},{}\n", log_time, string.as_ref()).expect("Cannot write to stdout");
    }

    pub fn log_msg<S: AsRef<str>>(&mut self, string: S) {
        write!(&mut self.stderr, "{}\n", string.as_ref()).expect("Cannot write to stderr");
    }
}
