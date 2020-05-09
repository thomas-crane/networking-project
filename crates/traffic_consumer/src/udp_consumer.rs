use crate::logger::Logger;
use std::net::UdpSocket;

pub struct UdpConsumer {
    logger: Logger,
}

impl UdpConsumer {
    pub fn new(logger: Logger) -> Self {
        Self { logger }
    }

    pub fn consume(&mut self) -> ! {
        let socket = UdpSocket::bind("0.0.0.0:6860").expect("Cannot create UDP socket");
        let mut buf = [0u8; 10_000];
        loop {
            let (bytes_received, from_addr) =
                socket.recv_from(&mut buf).expect("Cannot read from socket");

            println!(
                "Received {} bytes from {}",
                bytes_received,
                from_addr.to_string()
            );
            self.logger.log(&bytes_received.to_string());
        }
    }
}
