mod tcp_consumer;
mod udp_consumer;

use common::{logger::Logger, time::unix_time};
use crate::tcp_consumer::TcpConsumer;
use crate::udp_consumer::UdpConsumer;

fn main() {
    let mut args = std::env::args().skip(1);

    let mode = args.next().expect("Usage: consumer MODE");

    let logger = Logger::new(&format!("{}-{}.txt", unix_time(), mode));
    match mode.as_str() {
        "tcp" => {
            let mut consumer = TcpConsumer::new(logger);
            consumer.consume();
        }
        "udp" => {
            let mut consumer = UdpConsumer::new(logger);
            consumer.consume();
        }
        _ => panic!("No consumer for mode {}", mode),
    }
}
