use common::logger::Logger;
use protocol::lrdp_socket::LrdpSocket;
use throughput_recorder::snapshot::Snapshot;
use throughput_recorder::snapshot_taker::SnapshotTaker;

pub struct LrdpConsumer {
    logger: Logger,
    snapshot_taker: SnapshotTaker,
}

impl LrdpConsumer {
    pub fn new(logger: Logger) -> Self {
        let snapshot_taker = SnapshotTaker::new();
        Self {
            logger,
            snapshot_taker,
        }
    }

    pub fn consume(&mut self) -> () {
        let mut socket = LrdpSocket::bind("0.0.0.0:6860").expect("Cannot create LRDP socket");
        let mut recv_sum = 0;
        let mut packet_count = 0;
        // log initial snapshot.
        self.logger
            .log(format!("0,{},{}", recv_sum, self.snapshot().to_string()));

        loop {
            let (bytes_received, from_addr) = match socket.recv_from() {
                Ok((a, b)) => (a, b),
                _ => break,
            };
            if bytes_received.is_empty() {
                self.logger
                    .log_msg(format!("Received 0 bytes. Shutting down socket."));
                break;
            } else {
                self.logger.log_msg(format!(
                    "Received {} bytes from {}",
                    bytes_received.len(),
                    from_addr.to_string()
                ));
                recv_sum += bytes_received.len();
                packet_count += 1;
                self.logger.log(format!(
                    "{},{},{}",
                    packet_count,
                    recv_sum,
                    self.snapshot().to_string()
                ));
            }
        }
    }

    fn snapshot(&self) -> Snapshot {
        self.snapshot_taker
            .snapshot_of(&"eth0".to_string())
            .expect("Cannot get snapshot")
    }
}
