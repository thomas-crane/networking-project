use std::collections::{HashMap, VecDeque};
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

/// A socket which implements the "Semi Reliable Datagram Protocol".
pub struct SrdpSocket {
    // sockets
    read_socket: UdpSocket,
    shared_socket: Shared<UdpSocket>,

    // shared collections
    shared_available_ids: Shared<VecDeque<u8>>,
    shared_unacked_packets: Shared<HashMap<u8, UnackedPacket>>,

    // channels,
    packet_rx: mpsc::Receiver<(Vec<u8>, SocketAddr)>,
    packet_tx: mpsc::Sender<(Vec<u8>, SocketAddr)>,
}

impl SrdpSocket {
    /// Creates an `SrdpSocket` bound to the given `addrs`.
    pub fn bind<A: ToSocketAddrs>(addrs: A) -> Result<Self> {
        // set up sockets. `read_socket` can be `try_clone`d because it is only used for reading.
        // `shared_socket` has to be behind a mutex because it can be used for writing.
        let read_socket = UdpSocket::bind(addrs)?;
        let shared_socket = Arc::new(Mutex::new(read_socket.try_clone()?));

        // set up shared collections.
        let mut available_ids = VecDeque::with_capacity(64);
        for i in 0..64u8 {
            available_ids.push_front(i);
        }
        let shared_available_ids = Arc::new(Mutex::new(available_ids));
        let shared_unacked_packets = Arc::new(Mutex::new(HashMap::<u8, UnackedPacket>::new()));

        // set up the packet sender/receiver.
        let (packet_tx, packet_rx) = mpsc::channel();

        // reader thread
        // this thread has to:
        // 1. Read from the socket
        // 2. Parse the packet.
        // 3. If the packet was an ACK, remove the packet from the unacked list and put the id back
        //    into the list of available ids.
        // 4. If the packet was important, send an ACK for that packet.
        // 5. Send the packet to the socket mpsc.
        let thread_sockets = (read_socket.try_clone()?, shared_socket.clone());
        let thread_collections = (shared_available_ids.clone(), shared_unacked_packets.clone());
        let thread_sender = (packet_tx.clone(),);
        thread::spawn(move || {
            let (read_socket, write_socket) = thread_sockets;
            let (available_ids, unacked_packets) = thread_collections;
            let (sender,) = thread_sender;
            let mut buf = [0u8; std::u16::MAX as usize];
            loop {
                let (recv, addr) = read_socket.recv_from(&mut buf).unwrap();
                // close socket.
                if recv == 0 {
                    break;
                }
                // check flags
                let flags = buf[0] & FLAG_MASK;

                // check if it was an ack.
                if (flags & ACK_FLAG) == ACK_FLAG {
                    let id = buf[0] & ID_MASK;
                    let mut unacked_packets = unacked_packets.lock().unwrap();
                    // find the packet which was ACKed.
                    match unacked_packets.remove(&id) {
                        Some(_) => {
                            println!("Received ACK for {}", id);
                            // remove the acked packet and make the ID available again.
                            available_ids.lock().unwrap().push_back(id);
                        }
                        None => {
                            panic!("Tried to ACK packet that was already ACKed");
                        }
                    }
                    continue;
                }
                // check if we need to send an ack.
                if (flags & IMPORTANT_FLAG) == IMPORTANT_FLAG {
                    println!("Received important packet {}, sending ACK", buf[0] & ID_MASK);
                    // create an ack.
                    let ack = [ACK_FLAG | (buf[0] & ID_MASK)];
                    write_socket.lock().unwrap().send_to(&ack, addr).unwrap();
                }

                // dispatch the result.
                sender.send((buf[1..recv].to_vec(), addr)).unwrap();
            }
        });

        // ack watcher thread. runs at a given interval.
        // this thread has to:
        // 1. Go through the list of unacked packets.
        // 2. Check if the time since a packet was sent is past some threshold.
        // 3. If it is, send it again and update the time at which it was sent.
        let thread_sockets = (shared_socket.clone(),);
        let thread_collections = (shared_unacked_packets.clone(),);
        thread::spawn(move || {
            let (write_socket,) = thread_sockets;
            let (unacked_packets,) = thread_collections;

            loop {
                // wait a bit.
                thread::sleep(Duration::from_millis(10));
                let mut unacked_packets = unacked_packets.lock().unwrap();
                let now = Instant::now();
                for (_, packet) in unacked_packets.iter_mut() {
                    // check if 200ms has elapsed.
                    if now.duration_since(packet.last_sent).as_millis() > 200 {
                        println!("Sending {} again.", packet.id);
                        // send again.
                        let bytes = packet.as_bytes();
                        write_socket
                            .lock()
                            .unwrap()
                            .send_to(&bytes, packet.addr)
                            .unwrap();
                        packet.last_sent = now;
                    }
                }
            }
        });

        Ok(Self {
            read_socket,
            shared_socket,

            shared_available_ids,
            shared_unacked_packets,

            packet_tx,
            packet_rx,
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
                match self.shared_available_ids.lock().unwrap().pop_front() {
                    Some(id) => {
                        // add the packet to the unacked list.
                        let unacked = UnackedPacket::new(id, data, addr);
                        let bytes = unacked.as_bytes();
                        self.shared_unacked_packets
                            .lock()
                            .unwrap()
                            .insert(id, unacked);

                        // send the packet.
                        self.shared_socket.lock().unwrap().send_to(&bytes, addr)
                    }
                    None => {
                        // TODO
                        panic!("Exhausted available IDs.")
                    }
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
}

impl UnackedPacket {
    pub fn new(id: u8, data: Box<[u8]>, addr: SocketAddr) -> Self {
        Self {
            id,
            data,
            addr,
            last_sent: Instant::now(),
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
