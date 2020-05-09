use std::time::{SystemTime, UNIX_EPOCH};

pub fn unix_time() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Current time is earlier than epoch time")
        .as_secs()
        .to_string()
}
