use common::logger::Logger;
use std::io::Read;
use std::net::{Shutdown, TcpListener};
use throughput_recorder::snapshot::Snapshot;
use throughput_recorder::snapshot_taker::SnapshotTaker;

pub struct TcpConsumer {
    logger: Logger,
    snapshot_taker: SnapshotTaker,
}

impl TcpConsumer {
    pub fn new(logger: Logger) -> Self {
        let snapshot_taker = SnapshotTaker::new();
        Self {
            logger,
            snapshot_taker,
        }
    }

    pub fn consume(&mut self) -> () {
        let listener = TcpListener::bind("0.0.0.0:6860").expect("Cannot create TCP listener");
        let mut recv_sum = 0;
        // log initial snapshot.
        self.logger
            .log(&format!("{},{}", recv_sum, self.snapshot().to_string()));

        let (mut socket, from_addr) = listener.accept().expect("Cannot establish connection");
        println!("Received connection from {}", from_addr.to_string());

        let mut buf = [0u8; 10_000];
        loop {
            let bytes_received = socket.read(&mut buf).expect("Cannot read from socket");
            if bytes_received == 0 {
                println!("Received 0 bytes. Shutting down socket.");
                socket
                    .shutdown(Shutdown::Both)
                    .expect("Cannot shutdown socket");
                break;
            } else {
                println!("Received {} bytes", bytes_received);
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