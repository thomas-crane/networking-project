use std::fs::{File, OpenOptions};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Logger {
    file: File,
}

impl Logger {
    pub fn new(log_file: &String) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(log_file)
            .expect("Cannot open log file");

        Self { file }
    }

    pub fn log(&mut self, string: &String) {
        let log_time = self.time();
        write!(&mut self.file, "{},{}\n", log_time, string).expect("Cannot write to log file");
    }

    fn time(&self) -> String {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Current time is earlier than epoch time")
            .as_secs()
            .to_string()
    }
}
