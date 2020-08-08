/// The bitmask for the DATA flag in a packet.
const DATA_FLAG: u8 = 0b10000000;
/// The bitmask for the ACK flag in a packet.
const ACK_FLAG: u8  = 0b01000000;
/// The bitmask for the sequence number in a packet.
const SEQ_MASK: u8  = 0b00111000;
/// The bitmask for the acknowledgement number in a packet.
const ACK_MASK: u8  = 0b00000111;
/// The maximum possible sequence number.
const MAX_SEQ: u8   = 0b00000111;

/// A packet which conforms to the LRDP protocol.
pub struct LrdpPacket {
    has_ack: bool,
    has_data: bool,
    ack_num: u8,
    seq_num: u8,
    data: Box<[u8]>,
}

impl LrdpPacket {
    /// Create an LRDP packet from a received buffer.
    pub fn from_buffer(buf: &[u8]) -> Self {
        let has_ack = ACK_FLAG & buf[0] == ACK_FLAG;
        let has_data = DATA_FLAG & buf[0] == DATA_FLAG;
        let ack_num = ACK_MASK & buf[0];
        let seq_num = (SEQ_MASK & buf[0]) >> 3;
        Self {
            has_ack,
            has_data,
            ack_num,
            seq_num,
            data: buf[1..].into(),
        }
    }

    /// Create an LRDP packet from the given data and other details.
    ///
    /// + If `ack_num` is not `None`, this packet's ACK bit will be set.
    /// + If `seq_num` is not `None`, this packet's DATA bit will be set.
    pub fn create(data: Box<[u8]>, ack_num: Option<u8>, seq_num: Option<u8>) -> Self {
        Self {
            data,
            has_ack: ack_num.is_some(),
            ack_num: ack_num.unwrap_or(0),
            has_data: seq_num.is_some(),
            seq_num: seq_num.unwrap_or(0),
        }
    }

    /// Turn the packet into a buffer which can be sent over the network.
    pub fn as_buffer(&self) -> Vec<u8> {
        let mut buf = vec![0u8];

        // set flags.
        if self.has_ack() {
            buf[0] |= ACK_FLAG;
            buf[0] |= ACK_MASK & self.ack_num;
        }
        if self.has_data() {
            buf[0] |= DATA_FLAG;
            buf[0] |= (self.seq_num << 3) & SEQ_MASK;
            // extend with data if needed.
            buf.extend_from_slice(&self.data)
        }

        buf
    }

    /// Whether or not this packet's ACK bit is set.
    pub fn has_ack(&self) -> bool {
        self.has_ack
    }

    /// Whether or not this packet's DATA bit is set.
    pub fn has_data(&self) -> bool {
        self.has_data
    }

    /// The ACK number of this packet.
    pub fn ack_num(&self) -> u8 {
        self.ack_num
    }

    /// The SEQ number of this packet.
    pub fn seq_num(&self) -> u8 {
        self.seq_num
    }

    /// The data in this packet.
    pub fn data(&self) -> &Box<[u8]> {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_buffer_ack() {
        let packet = LrdpPacket::from_buffer(&[0b01000010]);
        assert!(packet.has_ack());
        assert_eq!(packet.ack_num(), 2);
    }

    #[test]
    fn test_from_buffer_data() {
        let packet = LrdpPacket::from_buffer(&[0b10110000, 1, 2, 3]);
        assert!(packet.has_data());
        assert_eq!(packet.seq_num(), 6);
        assert_eq!(packet.data.as_ref(), &[1, 2, 3]);
    }

    #[test]
    fn test_create() {
        let packet = LrdpPacket::create(Box::new([1, 2, 3]), Some(2), Some(3));
        assert_eq!(packet.data.as_ref(), &[1, 2, 3]);
        assert!(packet.has_data());
        assert!(packet.has_ack());
        assert_eq!(packet.ack_num(), 2);
        assert_eq!(packet.seq_num(), 3);
    }

    #[test]
    fn test_as_buffer() {
        let packet = LrdpPacket::create(Box::new([4, 5, 6]), None, Some(1));
        let buf = packet.as_buffer();
        assert_eq!(buf.as_slice(), &[0b10001000, 4, 5, 6]);
    }
}
