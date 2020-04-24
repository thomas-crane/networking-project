use crate::payload::create_payload;
use crate::producer::{Producer, ProducerOptions};
use std::io::Write;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::thread;
use std::time::Duration;

pub struct TcpProducer {
    destination: SocketAddr,
}

impl Producer for TcpProducer {
    fn new<T: ToSocketAddrs>(destination: T) -> Self {
        let addr = destination
            .to_socket_addrs()
            .ok()
            .and_then(|mut iter| iter.nth(0))
            .expect("Cannot convert into socket address");

        Self { destination: addr }
    }

    fn run(&self, opts: &ProducerOptions) {
        let mut socket = TcpStream::connect(self.destination).expect("Cannot create TCP socket");
        let delay_ms: u64 = (1000 / opts.rate).into();
        let mut payload = create_payload(opts.payload_size as usize);
        for i in 0..opts.count {
            socket.write_all(&mut payload).expect("Cannot send data");
            println!("Sent packet {} of {}", i + 1, opts.count);
            thread::sleep(Duration::from_millis(delay_ms));
        }
    }
}
