use std::thread;
use std::time::Duration;

use throughput_recorder::snapshot_logger::SnapshotLogger;
use throughput_recorder::snapshot_taker::SnapshotTaker;

fn main() {
    let mut args = std::env::args().skip(1);
    let if_name = args
        .next()
        .expect("Usage: throughput_recorder IF_NAME LOG_FILE [DELAY]");
    let log_file = args
        .next()
        .expect("Usage: throughput_recorder IF_NAME LOG_FILE [DELAY]");
    let delay_secs = args.next().and_then(|s| s.parse::<u64>().ok()).unwrap_or(1);

    let camera = SnapshotTaker::new();
    let mut logger = SnapshotLogger::new(log_file);
    loop {
        let snaps = camera.take_snapshot();
        let snap = snaps
            .iter()
            .find(|snap| snap.if_name == if_name)
            .unwrap_or_else(|| panic!("No interface with the name: {}", if_name));

        logger.append(&snap).expect("Couldnt append to log file");

        thread::sleep(Duration::from_secs(delay_secs));
    }
}
