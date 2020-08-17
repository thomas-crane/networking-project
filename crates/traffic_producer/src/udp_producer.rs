use crate::payload::create_payload;
use crate::producer::{Producer, ProducerRun};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::thread;
use std::time::Duration;

pub struct UdpProducer {
    destination: SocketAddr,
}

impl UdpProducer {
    pub fn new<T: ToSocketAddrs>(destination: T) -> Self {
        let addr = destination
            .to_socket_addrs()
            .ok()
            .and_then(|mut iter| iter.nth(0))
            .expect("Cannot convert into socket address");

        Self { destination: addr }
    }
}

impl Producer for UdpProducer {
    fn run(&self, runner: &mut ProducerRun) {
        // log an initial snapshot.
        let snapshot = runner.snapshot().to_string();
        runner.logger.log(format!("0,0,{}", snapshot));

        // since we are just sending stuff it doesn't really matter what we bind to.
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Cannot create UDP socket");
        let delay_ms: u64 = (1000.0 / runner.opts.rate) as u64;
        let payload = create_payload(runner.opts.payload_size as usize);
        let mut sent_sum = 0;
        for i in 0..runner.opts.count {
            runner
                .logger
                .log_msg(format!("Sending packet {} of {}", i + 1, runner.opts.count));
            socket
                .send_to(&payload, self.destination)
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
            .send_to(&[], self.destination)
            .expect("Cannot close socket");
    }
}
