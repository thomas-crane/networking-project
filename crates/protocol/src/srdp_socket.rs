use std::collections::VecDeque;
use std::io::Result;
use std::io::{Error, ErrorKind};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// A type which has been wrapped in an `Arc` and a `Mutex` for the purpose of being mutably used
/// in potentially many threads.
type Shared<T> = Arc<Mutex<T>>;

const FLAG_MASK: u8 = 0b11000000;
const ID_MASK: u8 = 0b00111111;
const IMPORTANT_FLAG: u8 = 0b10000000;
const ACK_FLAG: u8 = 0b01000000;
const EXPECT_FLAG: u8 = 0b11000000;

/// A socket which implements the "Semi Reliable Datagram Protocol".
pub struct SrdpSocket {
    // sockets
    read_socket: UdpSocket,
    shared_socket: Shared<UdpSocket>,

    // shared collections
    shared_unacked_packets: Shared<VecDeque<UnackedPacket>>,
    shared_send_times: Shared<VecDeque<u128>>,
    shared_ack_info: Shared<AckInfo>,

    // channels,
    packet_rx: mpsc::Receiver<(Vec<u8>, SocketAddr)>,
    packet_tx: mpsc::Sender<(Vec<u8>, SocketAddr)>,
    id_rx: mpsc::Receiver<u8>,
}

impl SrdpSocket {
    /// Creates an `SrdpSocket` bound to the given `addrs`.
    pub fn bind<A: ToSocketAddrs>(addrs: A) -> Result<Self> {
        // set up sockets. `read_socket` can be `try_clone`d because it is only used for reading.
        // `shared_socket` has to be behind a mutex because it can be used for writing.
        let read_socket = UdpSocket::bind(addrs)?;
        let shared_socket = Arc::new(Mutex::new(read_socket.try_clone()?));

        // set up shared collections.
        let shared_unacked_packets = Arc::new(Mutex::new(VecDeque::<UnackedPacket>::new()));
        let mut send_times = VecDeque::with_capacity(10);
        for _ in 0..send_times.capacity() {
            send_times.push_back(100);
        }
        let shared_send_times = Arc::new(Mutex::new(send_times));
        let shared_ack_info = Arc::new(Mutex::new(AckInfo::new()));

        // set up the channels.
        let (packet_tx, packet_rx) = mpsc::channel();
        let (id_tx, id_rx) = mpsc::channel();
        // enqueue all available packet ids.
        for i in 0..64u8 {
            id_tx.send(i).unwrap();
        }

        // reader thread
        // this thread has to:
        // 1. Read from the socket
        // 2. Parse the packet.
        // 3. If the packet was an ACK, remove the packet from the unacked list and put the id back
        //    into the list of available ids.
        // 4. If the packet was important, send an ACK for that packet.
        // 5. Send the packet to the socket mpsc.
        let thread_sockets = (read_socket.try_clone()?, shared_socket.clone());
        let thread_collections = (
            shared_unacked_packets.clone(),
            shared_send_times.clone(),
            shared_ack_info.clone(),
        );
        let thread_sender = (packet_tx.clone(), id_tx.clone());
        thread::spawn(move || {
            let mut seq_number = 0;
            let (read_socket, write_socket) = thread_sockets;
            let (unacked_packets, send_times, ack_info) = thread_collections;
            let (sender, id_queue) = thread_sender;
            let mut buf = [0u8; std::u16::MAX as usize];
            loop {
                let (recv, addr) = read_socket.recv_from(&mut buf).unwrap();
                // close socket.
                if recv == 0 {
                    break;
                }
                // check flags
                let flags = buf[0] & FLAG_MASK;

                // check if it was an expect.
                if (flags & EXPECT_FLAG) == EXPECT_FLAG {
                    let expected_id = buf[0] & ID_MASK;
                    // any unacked packets up until but not including the expected one can be
                    // removed.
                    let mut unacked_packets = unacked_packets.lock().unwrap();
                    loop {
                        let next_unacked_id = unacked_packets.front().map(|p| p.id);
                        match next_unacked_id {
                            Some(id) if id < expected_id => {
                                // remove this packet and make the ID available again.
                                unacked_packets.pop_front().unwrap();
                                id_queue.send(id).unwrap();
                            }
                            _ => break,
                        }
                    }
                    continue;
                }

                // check if it was an ack.
                if (flags & ACK_FLAG) == ACK_FLAG {
                    let acked_id = buf[0] & ID_MASK;
                    // remove any unacked packets up until and including the ACKed packet.
                    let mut unacked_packets = unacked_packets.lock().unwrap();
                    loop {
                        let next_unacked_id = unacked_packets.front().map(|p| p.id);
                        match next_unacked_id {
                            Some(id) if id <= acked_id => {
                                // remove this packet and make the ID available again.
                                let packet = unacked_packets.pop_front().unwrap();
                                id_queue.send(id).unwrap();
                                // when we remove the packet that was actually ACKed, also add its
                                // RTT to the list.
                                if packet.id == id {
                                    let rtt = Instant::now().duration_since(packet.first_sent);
                                    let mut send_times = send_times.lock().unwrap();
                                    send_times.pop_front();
                                    send_times.push_back(rtt.as_millis());
                                }
                            }
                            _ => break,
                        }
                    }
                    continue;
                }
                // check if we need to send an ack.
                if (flags & IMPORTANT_FLAG) == IMPORTANT_FLAG {
                    let id = buf[0] & ID_MASK;
                    // check if this packet was the next expected packet to be received. If it
                    // wasn't, sent an expect.
                    if id != seq_number {
                        let expect = [EXPECT_FLAG | seq_number];
                        write_socket.lock().unwrap().send_to(&expect, addr).unwrap();
                        continue;
                    } else {
                        // add one to the seq_number and make sure to wrap.
                        seq_number = (seq_number + 1) % 64;
                        let mut ack_info = ack_info.lock().unwrap();
                        if !ack_info.should_ack {
                            // if we weren't going to ack anything, set the necessary fields.
                            ack_info.should_ack = true;
                            ack_info.time = Instant::now();
                            ack_info.ack_number = id;
                            // FIXME: packets can arrive from different addresses, but all the acks
                            // will be sent to whatever address this is. Maybe use a hashmap of
                            // addresses instead.
                            ack_info.addr = Some(addr);
                        } else {
                            // otherwise, we received another packet before we had a chance to send
                            // an ACK, so we can just update the ack number.
                            ack_info.ack_number = id;
                        }
                        // acknowledge the packet.
                        // let ack = [ACK_FLAG | id];
                        // write_socket.lock().unwrap().send_to(&ack, addr).unwrap();
                    }
                }

                // dispatch the result minus the header.
                sender.send((buf[1..recv].to_vec(), addr)).unwrap();
            }
        });

        // ack watcher thread. runs at a given interval.
        // this thread has to:
        // 1. Go through the list of unacked packets.
        // 2. Check if the time since a packet was sent is past some threshold.
        // 3. If it is, send it again and update the time at which it was sent.
        let thread_sockets = (shared_socket.clone(),);
        let thread_collections = (
            shared_unacked_packets.clone(),
            shared_send_times.clone(),
            shared_ack_info.clone(),
        );
        thread::spawn(move || {
            let (write_socket,) = thread_sockets;
            let (unacked_packets, send_times, ack_info) = thread_collections;

            loop {
                // wait a bit.
                thread::sleep(Duration::from_millis(10));
                let avg_rtt = {
                    let send_times = send_times.lock().unwrap();
                    let rtt =
                        send_times.iter().fold(0, |acc, x| acc + x) / send_times.len() as u128;
                    // don't go lower than 10ms.
                    std::cmp::max(10, rtt)
                };
                // check if we need to ack anything
                let mut ack_info = ack_info.lock().unwrap();
                if ack_info.should_ack
                    && Instant::now().duration_since(ack_info.time).as_millis() > (avg_rtt / 3)
                {
                    // send the ack.
                    let header = ack_info.as_header();
                    write_socket
                        .lock()
                        .unwrap()
                        .send_to(&header, ack_info.addr.unwrap())
                        .unwrap();
                    ack_info.should_ack = false;
                }
                // release the lock.
                drop(ack_info);

                let mut unacked_packets = unacked_packets.lock().unwrap();
                // if there are no unacked packets there is nothing to resend.
                if unacked_packets.is_empty() {
                    continue;
                }
                for packet in unacked_packets.iter_mut() {
                    // check if the average RTT has elapsed.
                    if Instant::now().duration_since(packet.last_sent).as_millis() >= avg_rtt {
                        // send again.
                        let bytes = packet.as_bytes();
                        write_socket
                            .lock()
                            .unwrap()
                            .send_to(&bytes, packet.addr)
                            .unwrap();
                        packet.last_sent = Instant::now();
                    }
                }
            }
        });

        Ok(Self {
            read_socket,
            shared_socket,

            shared_unacked_packets,
            shared_send_times,
            shared_ack_info,

            packet_tx,
            packet_rx,
            id_rx,
        })
    }

    pub fn send_to(&mut self, packet: Packet, addr: SocketAddr) -> Result<usize> {
        match packet {
            // normal packet can just be sent.
            Packet::Normal(data) => {
                // add an empty header to the data.
                let message = [&[0u8], &*data].concat();
                self.shared_socket.lock().unwrap().send_to(&message, addr)
            }
            Packet::Important(data) => {
                // wait to receive an ID.
                match self.id_rx.recv() {
                    Ok(id) => {
                        // add the packet to the unacked queue.
                        let unacked = UnackedPacket::new(id, data, addr);
                        let bytes = unacked.as_bytes();
                        self.shared_unacked_packets
                            .lock()
                            .unwrap()
                            .push_back(unacked);

                        // send the packet.
                        self.shared_socket.lock().unwrap().send_to(&bytes, addr)
                    }
                    Err(_) => panic!("Cannot receive an ID."),
                }
            }
        }
    }

    pub fn recv(&self) -> Result<(Vec<u8>, SocketAddr)> {
        self.packet_rx
            .recv()
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }
}

#[derive(Debug)]
struct UnackedPacket {
    id: u8,
    data: Box<[u8]>,
    addr: SocketAddr,
    last_sent: Instant,
    first_sent: Instant,
}

impl UnackedPacket {
    pub fn new(id: u8, data: Box<[u8]>, addr: SocketAddr) -> Self {
        Self {
            id,
            data,
            addr,
            last_sent: Instant::now(),
            first_sent: Instant::now(),
        }
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        let header = [(self.id & ID_MASK) | IMPORTANT_FLAG];
        [&header, &*self.data].concat()
    }
}

pub enum Packet {
    Important(Box<[u8]>),
    Normal(Box<[u8]>),
}

struct AckInfo {
    should_ack: bool,
    time: Instant,
    ack_number: u8,
    addr: Option<SocketAddr>,
}

impl AckInfo {
    pub fn new() -> Self {
        Self {
            should_ack: false,
            time: Instant::now(),
            ack_number: 0,
            addr: None,
        }
    }
    pub fn as_header(&self) -> [u8; 1] {
        [(self.ack_number & ID_MASK) | ACK_FLAG]
    }
}
