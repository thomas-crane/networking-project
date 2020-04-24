use std::net::UdpSocket;

pub struct UdpConsumer;

impl UdpConsumer {
    pub fn new() -> Self {
        Self
    }

    pub fn consume(&self) -> ! {
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
        }
    }
}
