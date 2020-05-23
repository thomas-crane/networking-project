use crate::payload::create_payload;
use crate::producer::{Producer, ProducerRun};
use std::io::Write;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::thread;
use std::time::Duration;

pub struct TcpProducer {
    destination: SocketAddr,
}

impl TcpProducer {
    pub fn new<T: ToSocketAddrs>(destination: T) -> Self {
        let addr = destination
            .to_socket_addrs()
            .ok()
            .and_then(|mut iter| iter.nth(0))
            .expect("Cannot convert into socket address");

        Self { destination: addr }
    }
}

impl Producer for TcpProducer {
    fn run(&self, runner: &mut ProducerRun) {
        // log an initial snapshot.
        let snapshot = runner.snapshot().to_string();
        runner.logger.log(format!("0,0,{}", snapshot));

        let mut socket = TcpStream::connect(self.destination).expect("Cannot create TCP socket");
        let delay_ms: u64 = (1000 / runner.opts.rate).into();
        let mut sent_sum = 0;
        let mut payload = create_payload(runner.opts.payload_size as usize);
        for i in 0..runner.opts.count {
            runner
                .logger
                .log_msg(format!("Sending packet {} of {}", i + 1, runner.opts.count));
            socket.write_all(&mut payload).expect("Cannot send data");
            socket.flush().expect("Cannot flush stream");
            sent_sum += runner.opts.payload_size;
            // log the total packets sent, total bytes sent, and the current snapshot.
            let snapshot = runner.snapshot().to_string();
            runner
                .logger
                .log(format!("{},{},{}", i + 1, sent_sum, snapshot));
            thread::sleep(Duration::from_millis(delay_ms));
        }
    }
}
