use common::{logger::Logger, time::unix_time};
use std::fmt;
use throughput_recorder::snapshot::Snapshot;
use throughput_recorder::snapshot_taker::SnapshotTaker;

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
    pub fn as_filename(&self) -> String {
        format!("{}-{}-{}", self.count, self.rate, self.payload_size)
    }
}

impl fmt::Display for ProducerOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{}", self.count, self.rate, self.payload_size)
    }
}

pub trait Producer {
    fn name(&self) -> String;
    fn run(&self, runner: &mut ProducerRun) -> ();
}

pub struct ProducerRun {
    pub opts: ProducerOptions,
    pub logger: Logger,
    pub snapshot_taker: SnapshotTaker,
}

impl ProducerRun {
    pub fn new(opts: ProducerOptions, producer_name: String) -> Self {
        let snapshot_taker = SnapshotTaker::new();
        let logger = Logger::new(format!(
            "{}-{}-{}.txt",
            unix_time(),
            producer_name,
            opts.as_filename(),
        ));
        Self {
            opts,
            logger,
            snapshot_taker,
        }
    }

    pub fn run(&mut self, producer: Box<dyn Producer>) -> () {
        producer.run(self);
    }

    pub fn snapshot(&self) -> Snapshot {
        self.snapshot_taker
            .snapshot_of(&"eth0".to_string())
            .expect("Cannot get snapshot")
    }
}
