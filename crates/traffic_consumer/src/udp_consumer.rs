use common::logger::Logger;
use std::net::UdpSocket;
use throughput_recorder::snapshot::Snapshot;
use throughput_recorder::snapshot_taker::SnapshotTaker;

pub struct UdpConsumer {
    logger: Logger,
    snapshot_taker: SnapshotTaker,
}

impl UdpConsumer {
    pub fn new(logger: Logger) -> Self {
        let snapshot_taker = SnapshotTaker::new();
        Self {
            logger,
            snapshot_taker,
        }
    }

    pub fn consume(&mut self) -> () {
        let socket = UdpSocket::bind("0.0.0.0:6860").expect("Cannot create UDP socket");
        let mut buf = [0u8; 10_000];
        let mut recv_sum = 0;
        // log initial snapshot.
        self.logger
            .log(&format!("{},{}", recv_sum, self.snapshot().to_string()));

        loop {
            let (bytes_received, from_addr) =
                socket.recv_from(&mut buf).expect("Cannot read from socket");
            if bytes_received == 0 {
                println!("Received 0 bytes. Shutting down socket.");
                break;
            } else {
                println!(
                    "Received {} bytes from {}",
                    bytes_received,
                    from_addr.to_string()
                );
                recv_sum += bytes_received;
                self.logger
                    .log(&format!("{},{}", recv_sum, self.snapshot().to_string()));
            }
        }
    }

    fn snapshot(&self) -> Snapshot {
        self.snapshot_taker
            .take_snapshot()
            .into_iter()
            .nth(0)
            .expect("Cannot get snapshot")
    }
}
