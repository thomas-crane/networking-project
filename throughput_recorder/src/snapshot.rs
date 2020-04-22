use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct Snapshot {
    pub snapshot_time: u64,
    pub if_name: String,
    pub rx_bytes: u32,
    pub tx_bytes: u32,
}

impl Snapshot {
    fn new(if_name: String, rx_bytes: u32, tx_bytes: u32) -> Self {
        let snapshot_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Current time is earlier than epoch time")
            .as_secs();

        Self {
            snapshot_time,
            if_name,
            rx_bytes,
            tx_bytes,
        }
    }
}

impl fmt::Display for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{},{},{},{}",
            self.snapshot_time, self.if_name, self.rx_bytes, self.tx_bytes
        )
    }
}

// The file format is as follows.
//
// Inter-|   Receive                                                |  Transmit
// face  |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
// 0      1        2       3    4    5    6     7          8         9        10      11   12   13   14    15      16
//
// We're only interested an a few of the items in the vec.
impl From<Vec<&str>> for Snapshot {
    fn from(iter: Vec<&str>) -> Self {
        // get fields.
        let if_name = iter.get(0).expect("Missing interface name");
        let rx_bytes_str = iter.get(1).expect("Missing rx_bytes");
        let tx_bytes_str = iter.get(9).expect("Missing tx_bytes");

        // convert them into numbers.
        let rx_bytes = rx_bytes_str
            .parse::<u32>()
            .expect("rx_bytes was not a number");
        let tx_bytes = tx_bytes_str
            .parse::<u32>()
            .expect("tx_bytes was not a number");

        // the slice here is to cut off the `:` at the end of the name (which is not actually part
        // of the name).
        Self::new(
            if_name[0..if_name.len() - 1].to_string(),
            rx_bytes,
            tx_bytes,
        )
    }
}
