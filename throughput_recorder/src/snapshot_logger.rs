use std::error::Error;
use std::fs;
use std::io::Write;

use crate::snapshot::Snapshot;

pub struct SnapshotLogger {
    file: fs::File,
}

impl SnapshotLogger {
    pub fn new(log_file: String) -> Self {
        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .expect("Cannot open log file for appending");

        Self { file }
    }

    pub fn append(&mut self, snapshot: &Snapshot) -> Result<(), Box<dyn Error>> {
        self.file.write_all(snapshot.to_string().as_bytes())?;
        self.file.write_all(b"\n")?;
        Ok(())
    }
}
