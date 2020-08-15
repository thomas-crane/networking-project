use crate::lrdp_packet::LrdpPacket;
use log;
use std::collections::VecDeque;
use std::fmt;
use std::net::SocketAddr;
use std::time::Instant;

/// The maximum sequence number possible.
const MAX_SEQ: u8 = 8;

type ClientResult<T> = Result<T, ClientError>;

/// The state associated with a client connected over an LRDP socket.
pub struct ClientState {
    /// The address of this client.
    addr: SocketAddr,
    /// The sequence number of the received data.
    remote_seq: u8,
    /// The sequence number of the sent data.
    local_seq: u8,
    /// The last time something was received by this client.
    pub last_recv: Option<Instant>,
    /// The last time something was sent to this client.
    pub last_send: Option<Instant>,
    /// Packets which are waiting to be sent, or have been sent and not yet acknowledged.
    send_queue: VecDeque<LrdpPacket>,
}

impl ClientState {
    /// Creates a new client state associated with the given `addr`.
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            remote_seq: 0,
            local_seq: 0,
            last_recv: None,
            last_send: None,
            send_queue: VecDeque::with_capacity(8),
        }
    }

    /// Acknowledges all of the packets up to and including the one with the sequence number of
    /// `ack_num`.
    pub fn ack(&mut self, ack_num: u8) -> ClientResult<()> {
        log::trace!(target: &self.addr.to_string(), "Acking {}", ack_num);
        // make sure the ack number is actually in the queue.
        let has_num = self.send_queue.iter().any(|p| p.seq_num() == ack_num);
        if !has_num {
            // if the receiver was expecting the next number in the sequence, then this client is
            // just exhausted.
            if ack_num == self.local_seq {
                log::trace!(target: &self.addr.to_string(), "Client exhausted, draining queue ({} items).", self.send_queue.len());
                // drain the queue.
                while !self.send_queue.is_empty() {
                    self.send_queue.pop_front();
                }
                Err(ClientError::Exhausted)
            } else {
                log::trace!(target: &self.addr.to_string(), "Got wrong ack, local seq num is {}", self.local_seq);
                Err(ClientError::WrongAck(ack_num))
            }
        } else {
            // remove all packets until we reach the acked one.
            while let Some(packet) = self.send_queue.pop_front() {
                log::trace!(
                    target: &self.addr.to_string(),
                    "Removing packet {}",
                    packet.seq_num()
                );
                if packet.seq_num() == ack_num {
                    break;
                }
            }
            Ok(())
        }
    }

    /// Returns the packet at the front of the send queue.
    pub fn next_packet(&self) -> Option<&LrdpPacket> {
        self.send_queue.front()
    }

    /// Tries to receive the given sequence number. If the received sequence number is not the
    /// expected one, `ClientError::WrongSeq` will be returned.
    pub fn recv(&mut self, seq_num: u8) -> ClientResult<()> {
        if seq_num == self.remote_seq {
            self.remote_seq = (self.remote_seq + 1) % MAX_SEQ;
            Ok(())
        } else {
            Err(ClientError::WrongSeq(seq_num, self.remote_seq))
        }
    }

    /// Returns the next local sequence number.
    pub fn next_seq_num(&self) -> u8 {
        self.local_seq
    }

    /// Tries to add the `packet` to this client state's send queue. If the sequence number of the
    /// packet is not the expected one, `ClientError::WrongSeq` is returned.
    pub fn enqueue(&mut self, packet: LrdpPacket) -> ClientResult<()> {
        if packet.seq_num() != self.local_seq {
            Err(ClientError::WrongSeq(packet.seq_num(), self.local_seq))
        } else {
            self.send_queue.push_back(packet);
            self.local_seq = (self.local_seq + 1) % MAX_SEQ;
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub enum ClientError {
    /// The client received an acknowledgement number that it did not expect to receive. This most
    /// likely happens when the remote state is very far out of sync with the local state. In this
    /// situation there isn't much that can be done other than to bail.
    WrongAck(u8),
    /// The client received a sequence number that it did not expect to receive. The first element
    /// of this tuple is the received sequence number and the second element is the expected
    /// sequence number.
    WrongSeq(u8, u8),
    /// The receiver has acknowledged all of the data in the client's send queue, and there is no
    /// more data to send. This is caused when the client does not hear an ACK and thus retransmits
    /// the latest packet even though the receiver is expecting the next packet in the sequence.
    Exhausted,
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::WrongAck(a) => write!(
                f,
                "Received ACK number {} but there is no corresponding unacknowledged packet.",
                a
            ),
            Self::WrongSeq(actual, expected) => {
                write!(f, "Expected sequence number {}, got {}.", expected, actual)
            }
            Self::Exhausted => write!(f, "The client's send queue is exhausted."),
        }
    }
}

impl std::error::Error for ClientError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enqueue_good_seq_num() {
        let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), 0);
        let mut state = ClientState::new(addr);
        assert!(matches!(
            state.enqueue(LrdpPacket::create(Box::new([]), None, Some(0))),
            Ok(_)
        ));
    }

    #[test]
    fn enqueue_bad_seq_num() {
        let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), 0);
        let mut state = ClientState::new(addr);
        assert!(matches!(
            state.enqueue(LrdpPacket::create(Box::new([]), None, Some(2))),
            Err(ClientError::WrongSeq(2, 0))
        ));
    }

    #[test]
    fn ack_good_seq_num() {
        let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), 0);
        let mut state = ClientState::new(addr);
        state
            .enqueue(LrdpPacket::create(Box::new([]), None, Some(0)))
            .unwrap();
        assert!(matches!(state.ack(0), Ok(_)));
    }

    #[test]
    fn ack_bad_seq_num() {
        let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), 0);
        let mut state = ClientState::new(addr);
        state
            .enqueue(LrdpPacket::create(Box::new([]), None, Some(0)))
            .unwrap();
        assert!(matches!(state.ack(1), Err(ClientError::WrongAck(1))));
    }

    #[test]
    fn ack_mechanism_sequential() {
        let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), 0);
        let mut state = ClientState::new(addr);

        // enqueue a few packets.
        for i in 0..4 {
            state
                .enqueue(LrdpPacket::create(Box::new([]), None, Some(i)))
                .unwrap();
        }

        for i in 0..4 {
            // check that the expected packet is next in the queue.
            let packet = state.next_packet().unwrap();
            assert_eq!(packet.seq_num(), i);
            // ack it so that it is removed from the queue.
            state.ack(i).unwrap();
        }
    }

    #[test]
    fn ack_mechanism_delayed() {
        let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), 0);
        let mut state = ClientState::new(addr);

        // enqueue a few packets.
        for i in 0..4 {
            state
                .enqueue(LrdpPacket::create(Box::new([]), None, Some(i)))
                .unwrap();
        }

        // ack a whole bunch of the enqueued packets.
        state.ack(2).unwrap();

        // make sure the expected packet is next.
        let packet = state.next_packet().unwrap();
        assert_eq!(packet.seq_num(), 3);
    }
}
