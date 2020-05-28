use crate::payload::create_payload;
use crate::producer::{Producer, ProducerRun};
use protocol::srdp_socket::{Packet, SrdpSocket};
use std::net::{SocketAddr, ToSocketAddrs};
use std::thread;
use std::time::Duration;

pub struct SrdpProducer {
    destination: SocketAddr,
}

impl SrdpProducer {
    pub fn new<T: ToSocketAddrs>(destination: T) -> Self {
        let addr = destination
            .to_socket_addrs()
            .ok()
            .and_then(|mut iter| iter.nth(0))
            .expect("Cannot convert into socket address");

        Self { destination: addr }
    }
}

impl Producer for SrdpProducer {
    fn run(&self, runner: &mut ProducerRun) {
        // log an initial snapshot.
        let snapshot = runner.snapshot().to_string();
        runner.logger.log(format!("0,0,{}", snapshot));

        let mut socket = SrdpSocket::bind("0.0.0.0:0").expect("Cannot create SRDP socket");
        let delay_ms: u64 = (1000 / runner.opts.rate).into();
        let mut sent_sum = 0;
        for i in 0..runner.opts.count {
            runner
                .logger
                .log_msg(format!("Sending packet {} of {}", i + 1, runner.opts.count));
            let payload = create_payload(runner.opts.payload_size as usize);
            let packet = Packet::Important(payload.into_boxed_slice());
            socket
                .send_to(packet, self.destination)
                .expect("Cannot send data");
            sent_sum += runner.opts.payload_size;
            // log the total packets sent, total bytes sent, and the current snapshot.
            let snapshot = runner.snapshot().to_string();
            runner
                .logger
                .log(format!("{},{},{}", i + 1, sent_sum, snapshot));
            thread::sleep(Duration::from_millis(delay_ms));
        }
        // send a "closing" packet.
        socket
            .send_to(Packet::Normal(Box::new([])), self.destination)
            .expect("Cannot close socket");
    }
}
