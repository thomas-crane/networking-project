use crate::producer::ProducerOptions;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Logger {
    file: File,
}

impl Logger {
    pub fn new(file_name: &String) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(file_name)
            .expect("Cannot open file");

        Self { file }
    }

    pub fn log(&mut self, opts: &ProducerOptions) -> () {
        let log_time = self.time();
        write!(&mut self.file, "{},{}\n", log_time, opts).expect("Cannot write to log file");
    }

    fn time(&self) -> String {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Current time is earlier than epoch time")
            .as_secs()
            .to_string()
    }
}
