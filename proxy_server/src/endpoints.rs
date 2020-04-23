use std::env;
use std::net::{Ipv4Addr, SocketAddr};

pub struct Endpoints {
    producer_ip: Ipv4Addr,
    consumer_ip: Ipv4Addr,
}

impl Endpoints {
    pub fn new() -> Self {
        let producer_ip = env::var("PRODUCER_IP")
            .map(string_to_ip)
            .expect("No producer IP");
        let consumer_ip = env::var("CONSUMER_IP")
            .map(string_to_ip)
            .expect("No consumer IP");

        Self {
            producer_ip,
            consumer_ip,
        }
    }

    pub fn remote_endpoint_for(&self, endpoint: &SocketAddr) -> &Ipv4Addr {
        let ip = endpoint.ip();
        if ip == self.producer_ip {
            &self.consumer_ip
        } else if ip == self.consumer_ip {
            &self.producer_ip
        } else {
            panic!("No remote endpoint for {:?}", endpoint);
        }
    }
}

fn string_to_ip(string: String) -> Ipv4Addr {
    let mut parts = string
        .split('.')
        .map(|part| part.parse::<u8>().expect("Invalid IP address"));
    Ipv4Addr::new(
        parts.next().expect("Invalid IP address"),
        parts.next().expect("Invalid IP address"),
        parts.next().expect("Invalid IP address"),
        parts.next().expect("Invalid IP address"),
    )
}
