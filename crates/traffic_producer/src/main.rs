mod payload;
mod producer;
mod lrdp_producer;
mod tcp_producer;
mod udp_producer;

use crate::producer::{Producer, ProducerOptions, ProducerRun};
use crate::lrdp_producer::LrdpProducer;
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
        .and_then(|c| c.parse::<f32>().ok())
        .expect("Usage: producer MODE COUNT RATE PAYLOAD_SIZE");
    let payload_size = args
        .next()
        .and_then(|c| c.parse::<u32>().ok())
        .expect("Usage: producer MODE COUNT RATE PAYLOAD_SIZE");

    let proxy_ip = env::var("CONSUMER_IP").expect("No consumer IP");
    let addrs = format!("{}:6860", proxy_ip);

    let opts = ProducerOptions::new(count, rate, payload_size);

    let producer: Box<dyn Producer> = match mode.as_str() {
        "tcp" => Box::new(TcpProducer::new(&addrs)),
        "udp" => Box::new(UdpProducer::new(&addrs)),
        "lrdp" => Box::new(LrdpProducer::new(&addrs)),
        _ => panic!("Unsupported producer type"),
    };

    let mut runner = ProducerRun::new(opts);
    runner.run(producer);
}
