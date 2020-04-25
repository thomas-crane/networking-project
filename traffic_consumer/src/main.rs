mod logger;
mod tcp_consumer;
mod udp_consumer;

use crate::logger::Logger;
use crate::tcp_consumer::TcpConsumer;
use crate::udp_consumer::UdpConsumer;

fn main() {
    let mut args = std::env::args().skip(1);

    let mode = args.next().expect("Usage: consumer MODE LOG_FILE");
    let log_file = args.next().expect("Usage: consumer MODE LOG_FILE");

    let logger = Logger::new(&log_file);
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
