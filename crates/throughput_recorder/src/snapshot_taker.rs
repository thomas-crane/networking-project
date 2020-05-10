use crate::snapshot::Snapshot;
use std::process::Command;

pub struct SnapshotTaker;

impl SnapshotTaker {
    pub fn new() -> Self {
        Self
    }

    pub fn take_snapshot(&self) -> Vec<Snapshot> {
        let cmd = Command::new("cat")
            .arg("/proc/net/dev")
            .output()
            .expect("Cannot get output of cat /proc/net/dev");

        let result = String::from_utf8(cmd.stdout).expect("output was not utf-8");
        result
            .split('\n')
            .skip(2) // skip 2 because the header takes up 1 lines.
            .map(|line| line.split(' ')) // split each line into the words
            .map(|line| line.filter(|c| !c.is_empty())) // remove empty words (e.g. "")
            .map(|line| line.collect::<Vec<&str>>())
            .filter(|line| !line.is_empty())
            .map(|line| line.into())
            .collect::<Vec<Snapshot>>()
    }

    pub fn snapshot_of(&self, if_name: &String) -> Option<Snapshot> {
        self.take_snapshot()
            .into_iter()
            .find(|s| &s.if_name == if_name)
    }
}
