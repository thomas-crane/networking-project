use std::fs::{File, OpenOptions};
use std::path::Path;
use std::io::Write;
use crate::time::unix_time;

pub struct Logger {
    file: File,
}

impl Logger {
    /// Creates a new logger which will write logs into the given `log_file`.
    pub fn new<P: AsRef<Path>>(log_file: P) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(log_file)
            .expect("Cannot open log file");

        Self { file }
    }

    /// Appends both the current unix timestamp and the string to the file in CSV format, then adds
    /// a newline.
    pub fn log(&mut self, string: &String) {
        let log_time = unix_time();
        write!(&mut self.file, "{},{}\n", log_time, string).expect("Cannot write to log file");
    }
}
