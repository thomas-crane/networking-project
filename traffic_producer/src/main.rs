mod logger;
mod payload;
mod producer;
mod tcp_producer;
mod udp_producer;

use crate::producer::{Producer, ProducerOptions};
use crate::tcp_producer::TcpProducer;
use crate::udp_producer::UdpProducer;
use crate::logger::Logger;
use std::env;

fn main() {
    // get args.
    let mut args = std::env::args().skip(1);
    let mode = args
        .next()
        .expect("Usage: producer MODE COUNT RATE PAYLOAD_SIZE LOG");
    let count = args
        .next()
        .and_then(|c| c.parse::<u32>().ok())
        .expect("Usage: producer MODE COUNT RATE PAYLOAD_SIZE LOG");
    let rate = args
        .next()
        .and_then(|c| c.parse::<u32>().ok())
        .expect("Usage: producer MODE COUNT RATE PAYLOAD_SIZE LOG");
    let payload_size = args
        .next()
        .and_then(|c| c.parse::<u32>().ok())
        .expect("Usage: producer MODE COUNT RATE PAYLOAD_SIZE LOG");
    let log_file = args
        .next()
        .expect("Usage: producer MODE COUNT RATE PAYLOAD_SIZE LOG");

    let proxy_ip = env::var("PROXY_IP").expect("No proxy IP");
    let addrs = format!("{}:6850", proxy_ip);

    let opts = ProducerOptions::new(count, rate, payload_size);
    let mut logger = Logger::new(&log_file);

    logger.log(&opts);
    if mode == "tcp" {
        let producer = TcpProducer::new(&addrs);
        producer.run(&opts);
    }
    if mode == "udp" {
        let producer = UdpProducer::new(&addrs);
        producer.run(&opts);
    }
    logger.log(&opts);
}
