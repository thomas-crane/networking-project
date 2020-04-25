use std::io::Read;
use std::net::{Shutdown, TcpListener};
use crate::logger::Logger;

pub struct TcpConsumer {
    logger: Logger,
}

impl TcpConsumer {
    pub fn new(logger: Logger) -> Self {
        Self { logger }
    }

    pub fn consume(&mut self) -> () {
        let listener = TcpListener::bind("0.0.0.0:6860").expect("Cannot create TCP listener");

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
                self.logger.log(&bytes_received.to_string());
            }
        }
    }
}
