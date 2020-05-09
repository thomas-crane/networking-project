
mod endpoints;
mod server;
mod tcp_server;
mod udp_server;

use crate::server::string_to_server;

fn main() {
    let mut args = std::env::args().skip(1);
    let mode = args
        .next()
        .expect("Usage: proxy_server MODE");
    let server = string_to_server(mode);
    server.listen();
}
