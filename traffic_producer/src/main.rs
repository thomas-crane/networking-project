mod payload;
mod producer;
mod tcp_producer;
mod udp_producer;

use crate::producer::{Producer, ProducerRun, ProducerOptions};
use crate::tcp_producer::TcpProducer;
use crate::udp_producer::UdpProducer;
use std::env;

fn main() {
    // get args.
    let mut args = std::env::args().skip(1);
    let mode = args
        .next()
        .expect("Usage: producer MODE COUNT RATE PAYLOAD_SIZE");
    let count = args
        .next()
        .and_then(|c| c.parse::<u32>().ok())
        .expect("Usage: producer MODE COUNT RATE PAYLOAD_SIZE");
    let rate = args
        .next()
        .and_then(|c| c.parse::<u32>().ok())
        .expect("Usage: producer MODE COUNT RATE PAYLOAD_SIZE");
    let payload_size = args
        .next()
        .and_then(|c| c.parse::<u32>().ok())
        .expect("Usage: producer MODE COUNT RATE PAYLOAD_SIZE");

    let proxy_ip = env::var("PROXY_IP").expect("No proxy IP");
    let addrs = format!("{}:6850", proxy_ip);

    let opts = ProducerOptions::new(count, rate, payload_size);

    let producer: Box<dyn Producer> = match mode.as_str() {
        "tcp" => Box::new(TcpProducer::new(&addrs)),
        "udp" => Box::new(UdpProducer::new(&addrs)),
        _ => panic!("Unsupported producer type"),
    };

    let mut runner = ProducerRun::new(opts, producer.name());
    runner.run(producer);
}
