mod tcp_consumer;
mod udp_consumer;

use crate::tcp_consumer::TcpConsumer;
use crate::udp_consumer::UdpConsumer;

fn main() {
    let mut args = std::env::args().skip(1);

    let mode = args.next().expect("Usage: consumer MODE");

    match mode.as_str() {
        "tcp" => {
            let consumer = TcpConsumer::new();
            consumer.consume();
        }
        "udp" => {
            let consumer = UdpConsumer::new();
            consumer.consume();
        }
        _ => panic!("No consumer for mode {}", mode),
    }
}
