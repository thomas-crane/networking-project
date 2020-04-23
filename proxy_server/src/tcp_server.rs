use std::io::{Read, Write};
use std::net::{SocketAddrV4, TcpListener, TcpStream};
use std::thread;
use crate::endpoints::Endpoints;
use crate::server::Server;

pub struct TcpServer {
    endpoints: Endpoints,
    server: TcpListener,
}

impl TcpServer {
    pub fn new() -> Self {
        let server = TcpListener::bind("0.0.0.0:6850").expect("Cannot create TCP listener");
        let endpoints = Endpoints::new();

        Self { endpoints, server }
    }

}

impl Server for TcpServer {
    fn listen(&self) -> ! {
        // wait for a local connection.
        let (mut socket, from_addr) = self.server.accept().expect("Cannot establish local socket");

        // establish the remote connection. Make sure it goes to the same port.
        let remote_endpoint = self.endpoints.remote_endpoint_for(&from_addr);
        let remote_endpoint = SocketAddrV4::new(*remote_endpoint, from_addr.port());
        let mut remote_socket =
            TcpStream::connect(remote_endpoint).expect("Cannot establish remote connection");

        // remote socket reader.
        let mut remote_socket_clone = remote_socket
            .try_clone()
            .expect("Cannot clone remote socket");
        let mut socket_clone = socket.try_clone().expect("Cannot clone local socket");
        let remote_recv_thread = thread::spawn(move || {
            let mut recv_buf: [u8; 10_000] = [0u8; 10_000];
            loop {
                // read from remote.
                let bytes_read = remote_socket_clone
                    .read(&mut recv_buf)
                    .expect("Cannot read bytes from remote");
                // send to local.
                socket_clone
                    .write_all(&mut recv_buf[0..bytes_read])
                    .expect("Cannot write bytes to local");
            }
        });

        // local socket reader.
        let local_recv_thread = thread::spawn(move || {
            let mut recv_buf: [u8; 10_000] = [0u8; 10_000];
            loop {
                // read from local.
                let bytes_read = socket
                    .read(&mut recv_buf)
                    .expect("Cannot read bytes from local");

                // send to remote.
                remote_socket
                    .write_all(&mut recv_buf[0..bytes_read])
                    .expect("Cannot write bytes to remote");
            }
        });

        // wait for the threads to finish.
        local_recv_thread
            .join()
            .expect("Cannot join local recv thread");
        remote_recv_thread
            .join()
            .expect("Cannot join remote recv thread");

        panic!("Threads should not end")
    }
}
