mod srdp_consumer;
mod tcp_consumer;
mod udp_consumer;

use crate::srdp_consumer::SrdpConsumer;
use crate::tcp_consumer::TcpConsumer;
use crate::udp_consumer::UdpConsumer;
use common::logger::Logger;

fn main() {
    let mut args = std::env::args().skip(1);

    let mode = args.next().expect("Usage: consumer MODE");

    let logger = Logger::new();
    match mode.as_str() {
        "tcp" => {
            let mut consumer = TcpConsumer::new(logger);
            consumer.consume();
        }
        "udp" => {
            let mut consumer = UdpConsumer::new(logger);
            consumer.consume();
        }
        "srdp" => {
            let mut consumer = SrdpConsumer::new(logger);
            consumer.consume();
        }
        _ => panic!("No consumer for mode {}", mode),
    }
}
