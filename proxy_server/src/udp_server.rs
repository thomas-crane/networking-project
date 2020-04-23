use crate::endpoints::Endpoints;
use crate::server::Server;
use std::net::{SocketAddrV4, UdpSocket};

pub struct UdpServer {
    endpoints: Endpoints,
    socket: UdpSocket,
}

impl UdpServer {
    pub fn new() -> Self {
        let socket = UdpSocket::bind("0.0.0.0:6850").expect("Cannot create UDP server");
        let endpoints = Endpoints::new();

        Self { endpoints, socket }
    }
}

impl Server for UdpServer {
    fn listen(&self) -> ! {
        loop {
            let mut recv_buf: [u8; 10000] = [0u8; 10_000];
            let (bytes_received, from_addr) = self
                .socket
                .recv_from(&mut recv_buf)
                .expect("Couldn't receive data");

            let remote_endpoint = self.endpoints.remote_endpoint_for(&from_addr);
            let remote_endpoint = SocketAddrV4::new(*remote_endpoint, from_addr.port());
            self.socket
                .send_to(&recv_buf[0..bytes_received], remote_endpoint)
                .expect("Couldn't write to remote endpoint");
        }
    }
}
