use crate::tcp_server::TcpServer;
use crate::udp_server::UdpServer;

pub trait Server {
    fn listen(&self) -> !;
}

pub fn string_to_server(string: String) -> Box<dyn Server> {
    match string.as_str() {
        "tcp" => Box::new(TcpServer::new()),
        "udp" => Box::new(UdpServer::new()),
        _ => panic!("No server implementation for mode {}", string),
    }
}
