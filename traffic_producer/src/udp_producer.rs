use crate::payload::create_payload;
use crate::producer::{Producer, ProducerOptions};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::thread;
use std::time::Duration;

pub struct UdpProducer {
    destination: SocketAddr,
}

impl Producer for UdpProducer {
    fn new<T: ToSocketAddrs>(destination: T) -> Self {
        let addr = destination
            .to_socket_addrs()
            .ok()
            .and_then(|mut iter| iter.nth(0))
            .expect("Cannot convert into socket address");

        Self { destination: addr }
    }

    fn run(&self, opts: &ProducerOptions) {
        // since we are just sending stuff it doesn't really matter what we bind to.
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Cannot create UDP socket");
        let delay_ms: u64 = (1000 / opts.rate).into();
        let payload = create_payload(opts.payload_size as usize);
        for i in 0..opts.count {
            socket
                .send_to(&payload, self.destination)
                .expect("Cannot send data");
            println!("Sent packet {} of {}", i + 1, opts.count);
            thread::sleep(Duration::from_millis(delay_ms));
        }
    }
}
