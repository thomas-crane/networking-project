use std::fmt;
use std::net::ToSocketAddrs;

pub struct ProducerOptions {
    /// The number of packets to produce.
    pub count: u32,
    /// The rate in packets/sec at which traffic should be produced.
    pub rate: u32,
    /// The size in bytes of the payload for each packet.
    pub payload_size: u32,
}

impl ProducerOptions {
    pub fn new(count: u32, rate: u32, payload_size: u32) -> Self {
        Self {
            count,
            rate,
            payload_size,
        }
    }
}

impl fmt::Display for ProducerOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{}", self.count, self.rate, self.payload_size)
    }
}

pub trait Producer {
    fn new<T: ToSocketAddrs>(destination: T) -> Self;
    fn run(&self, options: &ProducerOptions) -> ();
}
