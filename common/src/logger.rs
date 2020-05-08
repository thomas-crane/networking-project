use std::fs::{File, OpenOptions};
use std::path::Path;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

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
        let log_time = self.time();
        write!(&mut self.file, "{},{}\n", log_time, string).expect("Cannot write to log file");
    }

    /// Gets the current unix time in seconds.
    fn time(&self) -> String {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Current time is earlier than epoch time")
            .as_secs()
            .to_string()
    }
}
